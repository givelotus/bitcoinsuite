import axios, { AxiosResponse } from "axios"
import WebSocket from "isomorphic-ws"
import * as ws from "ws"
import * as proto from "./chronik"
import { fromHex, toHex, toHexRev } from "./hex"

type MessageEvent = ws.MessageEvent | { data: Blob }

/** Client to access a Chronik instance.Plain object, without any
 * connections. */
export class ChronikClient {
  private _url: string
  private _wsUrl: string

  /**
   * Create a new client. This just creates an object, without any connections.
   *
   * @param url Url of a Chronik instance, with schema and without trailing
   *            slash. E.g. https://chronik.be.cash/xec.
   */
  constructor(url: string) {
    this._url = url
    if (url.endsWith("/")) {
      throw new Error("`url` cannot end with '/', got: " + url)
    }
    if (url.startsWith("https://")) {
      this._wsUrl = "wss://" + url.substring("https://".length)
    } else if (url.startsWith("http://")) {
      this._wsUrl = "ws://" + url.substring("http://".length)
    } else {
      throw new Error(
        "`url` must start with 'https://' or 'http://', got: " + url,
      )
    }
  }

  /** Broadcasts the `rawTx` on the network.
   * If `skipSlpCheck` is false, it will be checked that the tx doesn't burn
   * any SLP tokens before broadcasting.
   */
  public async broadcastTx(
    rawTx: Uint8Array | string,
  ): Promise<{ txid: string }> {
    const request = proto.BroadcastTxRequest.encode({
      rawTx: typeof rawTx === "string" ? fromHex(rawTx) : rawTx,
    }).finish()
    const data = await _post(this._url, "/broadcast-tx", request)
    const broadcastResponse = proto.BroadcastTxResponse.decode(data)
    return {
      txid: toHexRev(broadcastResponse.txid),
    }
  }

  /** Broadcasts the `rawTxs` on the network, only if all of them are valid.
   * If `skipSlpCheck` is false, it will be checked that the txs don't burn
   * any SLP tokens before broadcasting.
   */
  public async broadcastTxs(
    rawTxs: (Uint8Array | string)[],
  ): Promise<{ txids: string[] }> {
    const request = proto.BroadcastTxsRequest.encode({
      rawTxs: rawTxs.map(rawTx =>
        typeof rawTx === "string" ? fromHex(rawTx) : rawTx,
      ),
    }).finish()
    const data = await _post(this._url, "/broadcast-txs", request)
    const broadcastResponse = proto.BroadcastTxsResponse.decode(data)
    return {
      txids: broadcastResponse.txids.map(toHexRev),
    }
  }

  /** Fetch current info of the blockchain, such as tip hash and height. */
  public async blockchainInfo(): Promise<BlockchainInfo> {
    const data = await _get(this._url, `/blockchain-info`)
    const blockchainInfo = proto.BlockchainInfo.decode(data)
    return convertBlockchainInfo(blockchainInfo)
  }

  /** Fetch the block given hash or height. */
  public async block(hashOrHeight: string | number): Promise<Block> {
    const data = await _get(this._url, `/block/${hashOrHeight}`)
    const block = proto.Block.decode(data)
    return convertBlock(block)
  }

  /** Fetch block info of a range of blocks. `startHeight` and `endHeight` are
   * inclusive ranges. */
  public async blocks(
    startHeight: number,
    endHeight: number,
  ): Promise<BlockInfo[]> {
    const data = await _get(this._url, `/blocks/${startHeight}/${endHeight}`)
    const blocks = proto.Blocks.decode(data)
    return blocks.blocks.map(convertBlockInfo)
  }

  /** Fetch tx details given the txid. */
  public async tx(txid: string): Promise<Tx> {
    const data = await _get(this._url, `/tx/${txid}`)
    const tx = proto.Tx.decode(data)
    return convertTx(tx)
  }

  /** Fetch token info and stats given the tokenId. */
  public async tokenInfo(tokenId: string): Promise<TokenInfo> {
    const data = await _get(this._url, `/token-info/${tokenId}`)
    const token = proto.TokenInfo.decode(data)
    return convertTokenInfo(token)
  }

  /** Create object that allows fetching script history or UTXOs. */
  public script(scriptType: ScriptType, scriptPayload: string): ScriptEndpoint {
    return new ScriptEndpoint(this._url, scriptType, scriptPayload)
  }

  /** Open a WebSocket connection to listen for updates. */
  public ws(config: WsConfig): WsEndpoint {
    return new WsEndpoint(`${this._wsUrl}/ws`, config)
  }
}

/** Allows fetching script history and UTXOs. */
export class ScriptEndpoint {
  private _url: string
  private _scriptType: string
  private _scriptPayload: string

  constructor(url: string, scriptType: string, scriptPayload: string) {
    this._url = url
    this._scriptType = scriptType
    this._scriptPayload = scriptPayload
  }

  /** Fetches the tx history of this script, in anti-chronological order.
   * This means it's ordered by first-seen first. If the tx hasn't been seen
   * by the indexer before, it's ordered by the block timestamp.
   * @param page Page index of the tx history.
   * @param pageSize Number of txs per page.
   */
  public async history(
    page?: number,
    pageSize?: number,
  ): Promise<TxHistoryPage> {
    const query =
      page !== undefined && pageSize !== undefined
        ? `?page=${page}&page_size=${pageSize}`
        : page !== undefined
        ? `?page=${page}`
        : pageSize !== undefined
        ? `?page_size=${pageSize}`
        : ""
    const data = await _get(
      this._url,
      `/script/${this._scriptType}/${this._scriptPayload}/history${query}`,
    )
    const historyPage = proto.TxHistoryPage.decode(data)
    return {
      txs: historyPage.txs.map(convertTx),
      numPages: historyPage.numPages,
    }
  }

  /** Fetches the current UTXO set for this script.
   * It is grouped by output script, in case a script type can match multiple
   * different output scripts (e.g. Taproot on Lotus). */
  public async utxos(): Promise<ScriptUtxos> {
    const data = await _get(
      this._url,
      `/script/${this._scriptType}/${this._scriptPayload}/utxos`,
    )
    const utxos = proto.ScriptUtxos.decode(data)
    return {
      script: toHex(utxos.script),
      utxos: utxos.utxos.map(convertUtxo),
    }
  }
}

/** Config for a WebSocket connection to Chronik. */
export interface WsConfig {
  /** Fired when a message is sent from the WebSocket. */
  onMessage?: (msg: WsMsg) => void

  /** Fired when a connection has been (re)established. */
  onConnect?: (e: ws.Event) => void

  /** Fired after a connection has been unexpectedly closed, and before a
   * reconnection attempt is made. Only fired if `autoReconnect` is true. */
  onReconnect?: (e: ws.Event) => void

  /** Fired when an error with the WebSocket occurs. */
  onError?: (e: ws.ErrorEvent) => void

  /** Fired after a connection has been manually closed, or if `autoReconnect`
   * is false, if the WebSocket disconnects for any reason. */
  onEnd?: (e: ws.Event) => void

  /** Whether to automatically reconnect on disconnect, default true. */
  autoReconnect?: boolean
}

/** WebSocket connection to Chronik. */
export class WsEndpoint {
  /** Fired when a message is sent from the WebSocket. */
  public onMessage?: (msg: WsMsg) => void

  /** Fired when a connection has been (re)established. */
  public onConnect?: (e: ws.Event) => void

  /** Fired after a connection has been unexpectedly closed, and before a
   * reconnection attempt is made. Only fired if `autoReconnect` is true. */
  public onReconnect?: (e: ws.Event) => void

  /** Fired when an error with the WebSocket occurs. */
  public onError?: (e: ws.ErrorEvent) => void

  /** Fired after a connection has been manually closed, or if `autoReconnect`
   * is false, if the WebSocket disconnects for any reason. */
  public onEnd?: (e: ws.Event) => void

  /** Whether to automatically reconnect on disconnect, default true. */
  public autoReconnect: boolean

  private _ws: ws.WebSocket | undefined
  private _wsUrl: string
  private _connected: Promise<ws.Event> | undefined
  private _manuallyClosed: boolean
  private _subs: { scriptType: ScriptType; scriptPayload: string }[]

  constructor(wsUrl: string, config: WsConfig) {
    this.onMessage = config.onMessage
    this.onConnect = config.onConnect
    this.onReconnect = config.onReconnect
    this.onEnd = config.onEnd
    this.autoReconnect =
      config.autoReconnect !== undefined ? config.autoReconnect : true
    this._manuallyClosed = false
    this._subs = []
    this._wsUrl = wsUrl
    this._connect()
  }

  /** Wait for the WebSocket to be connected. */
  public async waitForOpen() {
    await this._connected
  }

  /** Subscribe to the given script type and payload.
   * For "p2pkh", `scriptPayload` is the 20 byte public key hash. */
  public subscribe(scriptType: ScriptType, scriptPayload: string) {
    this._subs.push({ scriptType, scriptPayload })
    if (this._ws?.readyState === WebSocket.OPEN) {
      this._subUnsub(false, scriptType, scriptPayload)
    }
  }

  /** Unsubscribe from the given script type and payload. */
  public unsubscribe(scriptType: ScriptType, scriptPayload: string) {
    this._subs = this._subs.filter(
      sub =>
        sub.scriptType !== scriptType || sub.scriptPayload !== scriptPayload,
    )
    if (this._ws?.readyState === WebSocket.OPEN) {
      this._subUnsub(true, scriptType, scriptPayload)
    }
  }

  /** Close the WebSocket connection and prevent future any reconnection
   * attempts. */
  public close() {
    this._manuallyClosed = true
    this._ws?.close()
  }

  private _connect() {
    const ws: ws.WebSocket = new WebSocket(this._wsUrl)
    this._ws = ws
    this._connected = new Promise(resolved => {
      ws.onopen = msg => {
        this._subs.forEach(sub =>
          this._subUnsub(false, sub.scriptType, sub.scriptPayload),
        )
        resolved(msg)
        if (this.onConnect !== undefined) {
          this.onConnect(msg)
        }
      }
    })
    ws.onmessage = e => this._handleMsg(e as MessageEvent)
    ws.onerror = e => this.onError !== undefined && this.onError(e)
    ws.onclose = e => {
      // End if manually closed or no auto-reconnect
      if (this._manuallyClosed || !this.autoReconnect) {
        if (this.onEnd !== undefined) {
          this.onEnd(e)
        }
        return
      }
      if (this.onReconnect !== undefined) {
        this.onReconnect(e)
      }
      this._connect()
    }
  }

  private _subUnsub(
    isUnsub: boolean,
    scriptType: ScriptType,
    scriptPayload: string,
  ) {
    const encodedSubscription = proto.WsSub.encode(<proto.WsSub>{
      isUnsub,
      script: {
        scriptType,
        payload: fromHex(scriptPayload),
      },
    }).finish()
    if (this._ws === undefined)
      throw new Error("Invalid state; _ws is undefined")
    this._ws.send(encodedSubscription)
  }

  private async _handleMsg(wsMsg: MessageEvent) {
    if (this.onMessage === undefined) {
      return
    }
    const data =
      wsMsg.data instanceof Buffer
        ? (wsMsg.data as Uint8Array)
        : new Uint8Array(await (wsMsg.data as Blob).arrayBuffer())
    const msg = proto.WsMsg.decode(data)
    if (msg.error) {
      this.onMessage({
        type: "Error",
        ...msg.error,
      })
    } else if (msg.tx) {
      const txMsgType = ((msgType: proto.TxMsgType) => {
        switch (msgType) {
          case proto.TxMsgType.TX_ADDED_TO_MEMPOOL:
            return <const>"AddedToMempool"
          case proto.TxMsgType.TX_REMOVED_FROM_MEMPOOL:
            return <const>"RemovedFromMempool"
          case proto.TxMsgType.TX_CONFIRMED:
            return <const>"Confirmed"
          case proto.TxMsgType.TX_FINALIZED:
            return <const>"Finalized"
        }
      })(msg.tx.msgType)
      if (txMsgType === undefined) {
        console.log("Silently ignored unknown Chronik tx message:", msg)
        return
      }
      this.onMessage({
        type: "MsgTx",
        txMsgType,
        txid: toHexRev(msg.tx.txid),
      })
    } else if (msg.block) {
      const blockMsgType = ((msgType: proto.BlockMsgType) => {
        switch (msgType) {
          case proto.BlockMsgType.BLK_CONNECTED:
            return <const>"Connected"
          case proto.BlockMsgType.BLK_DISCONNECTED:
            return <const>"Disconnected"
          case proto.BlockMsgType.BLK_FINALIZED:
            return <const>"Finalized"
        }
      })(msg.block.msgType)
      if (blockMsgType === undefined) {
        console.log("Silently ignored unknown Chronik block message:", msg)
        return
      }
      this.onMessage({
        type: "MsgBlock",
        blockMsgType,
        blockHash: toHexRev(msg.block.blockHash),
        blockHeight: msg.block.blockHeight,
      })
    } else {
      console.log("Silently ignored unknown Chronik message:", msg)
    }
  }
}

async function _get(url: string, path: string): Promise<Uint8Array> {
  const response = await axios.get(`${url}${path}`, {
    responseType: "arraybuffer",
    validateStatus: undefined,
  })
  ensureResponseErrorThrown(response, path)
  return new Uint8Array(response.data)
}

async function _post(
  url: string,
  path: string,
  data: Uint8Array,
): Promise<Uint8Array> {
  const response = await axios.post(`${url}${path}`, data, {
    responseType: "arraybuffer",
    validateStatus: undefined,
    // Prevents Axios encoding the Uint8Array as JSON or something
    transformRequest: x => x,
    headers: {
      "Content-Type": "application/x-protobuf",
    },
  })
  ensureResponseErrorThrown(response, path)
  return new Uint8Array(response.data)
}

function ensureResponseErrorThrown(response: AxiosResponse, path: string) {
  if (response.status != 200) {
    const error = proto.Error.decode(new Uint8Array(response.data))
    throw new Error(`Failed getting ${path}: ${error.msg}`)
  }
}

function convertBlockchainInfo(
  blockchainInfo: proto.BlockchainInfo,
): BlockchainInfo {
  return {
    tipHash: toHexRev(blockchainInfo.tipHash),
    tipHeight: blockchainInfo.tipHeight,
  }
}

function convertBlock(block: proto.Block): Block {
  if (block.blockInfo === undefined) {
    throw new Error("Block has no blockInfo")
  }
  return {
    blockInfo: convertBlockInfo(block.blockInfo),
  }
}

function convertTx(tx: proto.Tx): Tx {
  return {
    txid: toHexRev(tx.txid),
    version: tx.version,
    inputs: tx.inputs.map(convertTxInput),
    outputs: tx.outputs.map(convertTxOutput),
    lockTime: tx.lockTime,
    block: tx.block !== undefined ? convertBlockMeta(tx.block) : undefined,
    timeFirstSeen: tx.timeFirstSeen,
    size: tx.size,
    isCoinbase: tx.isCoinbase,
    slpv1Data:
      tx.slpv1Data !== undefined ? convertSlpv1TxData(tx.slpv1Data) : undefined,
    slpv2Sections: tx.slpv2Sections.map(convertSlpv2Section),
    slpBurns: tx.slpBurns.map(convertSlpBurn),
    slpErrors: tx.slpErrors,
  }
}

function convertUtxo(utxo: proto.ScriptUtxo): ScriptUtxo {
  if (utxo.outpoint === undefined) {
    throw new Error("UTXO outpoint is undefined")
  }
  return {
    outpoint: {
      txid: toHexRev(utxo.outpoint.txid),
      outIdx: utxo.outpoint.outIdx,
    },
    blockHeight: utxo.blockHeight,
    isCoinbase: utxo.isCoinbase,
    value: utxo.value,
    isFinal: utxo.isFinal,
    slp: utxo.slp !== undefined ? convertSlpToken(utxo.slp) : undefined,
  }
}

function convertTxInput(input: proto.TxInput): TxInput {
  if (input.prevOut === undefined) {
    throw new Error("Invalid proto, no prevOut")
  }
  return {
    prevOut: {
      txid: toHexRev(input.prevOut.txid),
      outIdx: input.prevOut.outIdx,
    },
    inputScript: toHex(input.inputScript),
    outputScript:
      input.outputScript.length > 0 ? toHex(input.outputScript) : undefined,
    value: input.value,
    sequenceNo: input.sequenceNo,
    slp: input.slp !== undefined ? convertSlpToken(input.slp) : undefined,
  }
}

function convertTxOutput(output: proto.TxOutput): TxOutput {
  return {
    value: output.value,
    outputScript: toHex(output.outputScript),
    spentBy:
      output.spentBy !== undefined
        ? {
            txid: toHexRev(output.spentBy.txid),
            inputIdx: output.spentBy.inputIdx,
          }
        : undefined,
    slp: output.slp !== undefined ? convertSlpToken(output.slp) : undefined,
  }
}

function convertSlpv1TxData(slpTxData: proto.Slpv1TxData): Slpv1TxData {
  return {
    tokenType: convertSlpv1TokenType(slpTxData.tokenType),
    txType: convertSlpTxType(slpTxData.txType),
    tokenId: toHex(slpTxData.tokenId),
    groupTokenId:
      slpTxData.groupTokenId.length == 32
        ? toHex(slpTxData.groupTokenId)
        : undefined,
  }
}

function convertSlpv2Section(section: proto.Slpv2Section): Slpv2Section {
  return {
    tokenId: toHexRev(section.tokenId),
    tokenType: convertSlpv2TokenType(section.tokenType),
    sectionType: convertSlpTxType(section.sectionType),
  }
}

function convertTokenProtocol(
  tokenProtocol: proto.TokenProtocol,
): TokenProtocol {
  switch (tokenProtocol) {
    case proto.TokenProtocol.TOKEN_PROTOCOL_SLPV1:
      return "SLP"
    case proto.TokenProtocol.TOKEN_PROTOCOL_SLPV2:
      return "SLPV2"
    default:
      throw new Error(`Invalid token protocol: ${tokenProtocol}`)
  }
}

function convertSlpv1TokenType(
  tokenType: proto.Slpv1TokenType,
): Slpv1TokenType {
  switch (tokenType) {
    case proto.Slpv1TokenType.SLPV1_TOKEN_TYPE_FUNGIBLE:
      return "FUNGIBLE"
    case proto.Slpv1TokenType.SLPV1_TOKEN_TYPE_NFT1_GROUP:
      return "NFT1_GROUP"
    case proto.Slpv1TokenType.SLPV1_TOKEN_TYPE_NFT1_CHILD:
      return "NFT1_CHILD"
    case proto.Slpv1TokenType.SLPV1_TOKEN_TYPE_UNKNOWN:
      return "UNKNOWN"
    default:
      throw new Error(`Invalid SLP v1 token type: ${tokenType}`)
  }
}

function convertSlpv2TokenType(
  tokenType: proto.Slpv2TokenType,
): Slpv2TokenType {
  switch (tokenType) {
    case proto.Slpv2TokenType.SLPV2_TOKEN_TYPE_STANDARD:
      return "STANDARD"
    case proto.Slpv2TokenType.UNRECOGNIZED:
      return "UNKNOWN"
    default:
      throw new Error(`Invalid SLPv2 token type: ${tokenType}`)
  }
}

function convertSlpTxType(txType: proto.SlpTxType): SlpTxType {
  switch (txType) {
    case proto.SlpTxType.GENESIS:
      return "GENESIS"
    case proto.SlpTxType.SEND:
      return "SEND"
    case proto.SlpTxType.MINT:
      return "MINT"
    case proto.SlpTxType.BURN:
      return "BURN"
    case proto.SlpTxType.UNKNOWN:
      return "UNKNOWN"
    default:
      throw new Error(`Invalid slp tx type: ${txType}`)
  }
}

function convertTokenInfo(info: proto.TokenInfo): TokenInfo {
  const tokenProtocol = convertTokenProtocol(info.tokenProtocol)
  return {
    tokenId:
      tokenProtocol == "SLP" ? toHex(info.tokenId) : toHexRev(info.tokenId),
    tokenProtocol,
    slpv1:
      tokenProtocol === "SLP"
        ? {
            tokenType: convertSlpv1TokenType(info.slpv1TokenType),
            genesisInfo: convertSlpv1GenesisInfo(info.slpv1GenesisInfo!),
          }
        : undefined,
    slpv2:
      tokenProtocol === "SLPV2"
        ? {
            tokenType: convertSlpv2TokenType(info.slpv2TokenType),
            genesisInfo: convertSlpv2GenesisInfo(info.slpv2GenesisInfo!),
          }
        : undefined,
    block: info.block !== undefined ? convertBlockMeta(info.block) : undefined,
    timeFirstSeen: info.timeFirstSeen,
  }
}

function convertSlpv1GenesisInfo(
  info: proto.Slpv1GenesisInfo,
): Slpv1GenesisInfo {
  const decoder = new TextDecoder()
  return {
    tokenTicker: decoder.decode(info.tokenTicker),
    tokenName: decoder.decode(info.tokenName),
    tokenDocumentUrl: decoder.decode(info.tokenDocumentUrl),
    tokenDocumentHash: toHex(info.tokenDocumentHash),
    decimals: info.decimals,
  }
}

function convertSlpv2GenesisInfo(
  info: proto.Slpv2GenesisInfo,
): Slpv2GenesisInfo {
  const decoder = new TextDecoder()
  return {
    tokenTicker: decoder.decode(info.tokenTicker),
    tokenName: decoder.decode(info.tokenName),
    url: decoder.decode(info.url),
    data: info.data,
    authPubkey: toHex(info.authPubkey),
    decimals: info.decimals,
  }
}

function convertBlockMeta(block: proto.BlockMetadata): BlockMetadata {
  return {
    height: block.height,
    hash: toHexRev(block.hash),
    timestamp: block.timestamp,
    isFinal: block.isFinal,
  }
}

function convertBlockInfo(block: proto.BlockInfo): BlockInfo {
  return {
    ...block,
    hash: toHexRev(block.hash),
    prevHash: toHexRev(block.prevHash),
  }
}

function convertSlpBurn(burn: proto.SlpBurn): SlpBurn {
  const tokenProtocol = convertTokenProtocol(burn.tokenProtocol)
  return {
    tokenId:
      tokenProtocol == "SLP" ? toHex(burn.tokenId) : toHexRev(burn.tokenId),
    tokenProtocol,
    slpv1TokenType:
      tokenProtocol === "SLP"
        ? convertSlpv1TokenType(burn.slpv1TokenType)
        : undefined,
    slpv2TokenType:
      tokenProtocol === "SLPV2"
        ? convertSlpv2TokenType(burn.slpv2TokenType)
        : undefined,
    burnError: burn.burnError,
    actualBurn:
      tokenProtocol === "SLP" ? burn.slpv1ActualBurn : burn.slpv2ActualBurn,
    slpv2IntentionalBurn: burn.slpv2IntentionalBurn,
    burnMintBatons: burn.burnMintBatons,
  }
}

function convertSlpToken(token: proto.SlpToken): SlpToken {
  const tokenProtocol = convertTokenProtocol(token.tokenProtocol)
  return {
    tokenId:
      tokenProtocol == "SLP" ? toHex(token.tokenId) : toHexRev(token.tokenId),
    tokenProtocol,
    slpv1TokenType:
      tokenProtocol === "SLP"
        ? convertSlpv1TokenType(token.slpv1TokenType)
        : undefined,
    slpv2TokenType:
      tokenProtocol === "SLPV2"
        ? convertSlpv2TokenType(token.slpv2TokenType)
        : undefined,
    slpv2SectionIdx: token.slpv2SectionIdx,
    isBurned: token.isBurned,
    amount: token.amount,
    isMintBaton: token.isMintBaton,
  }
}

/** Current state of the blockchain. */
export interface BlockchainInfo {
  /** Block hash of the current blockchain tip */
  tipHash: string
  /** Current height of the blockchain */
  tipHeight: number
}

/** A transaction on the blockchain or in the mempool. */
export interface Tx {
  /** Transaction ID.
   * - On BCH, eCash and Ergon, this is the hash of the tx.
   * - On Lotus, this is a special serialization, omitting the input scripts.
   */
  txid: string
  /** `version` field of the transaction. */
  version: number
  /** Inputs of this transaction. */
  inputs: TxInput[]
  /** Outputs of this transaction. */
  outputs: TxOutput[]
  /** `locktime` field of the transaction, tx is not valid before this time. */
  lockTime: number
  /** Block data for this tx, or undefined if not mined yet. */
  block: BlockMetadata | undefined
  /** UNIX timestamp when this tx has first been seen in the mempool.
   * 0 if unknown -> make sure to check.
   */
  timeFirstSeen: string
  /** Serialized size of the tx. */
  size: number
  /** Whether this tx is a coinbase tx. */
  isCoinbase: boolean
  slpv1Data: Slpv1TxData | undefined
  slpv2Sections: Slpv2Section[]
  slpBurns: SlpBurn[]
  slpErrors: string[]
}

/** An unspent transaction output (aka. UTXO, aka. "Coin") of a script. */
export interface ScriptUtxo {
  /** Outpoint of the UTXO. */
  outpoint: OutPoint
  /** Which block this UTXO is in, or -1 if in the mempool. */
  blockHeight: number
  /** Whether this UTXO is a coinbase UTXO
   * (make sure it's buried 100 blocks before spending!) */
  isCoinbase: boolean
  /** Value of the UTXO in satoshis. */
  value: string
  isFinal: boolean
  /** SLP and SLPv2 data in this UTXO. */
  slp: SlpToken | undefined
}

/** Block info about a block */
export interface BlockInfo {
  /** Block hash of the block, in 'human-readable' (big-endian) hex encoding. */
  hash: string
  /** Block hash of the previous block, in 'human-readable' (big-endian) hex
   * encoding. */
  prevHash: string
  /** Height of the block; Genesis block has height 0. */
  height: number
  /** nBits field of the block, encodes the target compactly. */
  nBits: number
  /** Timestamp of the block. Filled in by the miner, so might not be 100%
   * precise. */
  timestamp: string
  /** Block size of this block in bytes (including headers etc.). */
  blockSize: string
  /** Number of txs in this block. */
  numTxs: string
  /** Total number of tx inputs in block (including coinbase). */
  numInputs: string
  /** Total number of tx output in block (including coinbase). */
  numOutputs: string
  /** Total number of satoshis spent by tx inputs. */
  sumInputSats: string
  /** Total block reward for this block. */
  sumCoinbaseOutputSats: string
  /** Total number of satoshis in non-coinbase tx outputs. */
  sumNormalOutputSats: string
  /** Total number of satoshis burned using OP_RETURN. */
  sumBurnedSats: string
  isFinal: boolean
}

/** Block on the blockchain. */
export interface Block {
  /** Info about the block. */
  blockInfo: BlockInfo
}

/** Group of UTXOs by output script. */
export interface ScriptUtxos {
  /** Output script in hex. */
  script: string
  /** UTXOs of the output script. */
  utxos: ScriptUtxo[]
}

/** Page of the transaction history. */
export interface TxHistoryPage {
  /** Txs of this page. */
  txs: Tx[]
  /** Number of pages of the entire transaction history.
   * This changes based on the `pageSize` provided. */
  numPages: number
}

export type TokenProtocol = "SLP" | "SLPV2"

/** Which SLP tx type or SLPv2 section type. */
export type SlpTxType = "GENESIS" | "SEND" | "MINT" | "BURN" | "UNKNOWN"

/** Which SLP token type (normal fungible, NFT, unknown). */
export type Slpv1TokenType =
  | "FUNGIBLE"
  | "NFT1_GROUP"
  | "NFT1_CHILD"
  | "UNKNOWN"

export type Slpv2TokenType = "STANDARD" | "UNKNOWN"

export interface TokenInfo {
  tokenId: string
  tokenProtocol: TokenProtocol
  slpv1:
    | {
        tokenType: Slpv1TokenType
        genesisInfo: Slpv1GenesisInfo
      }
    | undefined
  slpv2:
    | {
        tokenType: Slpv2TokenType
        genesisInfo: Slpv2GenesisInfo
      }
    | undefined
  block: BlockMetadata | undefined
  timeFirstSeen: string
}

export interface Slpv1TxData {
  tokenType: Slpv1TokenType
  txType: SlpTxType
  tokenId: string
  groupTokenId: string | undefined
}

export interface Slpv2Section {
  tokenId: string
  tokenType: Slpv2TokenType
  sectionType: SlpTxType
}

export interface SlpBurn {
  tokenId: string
  tokenProtocol: TokenProtocol
  slpv1TokenType: Slpv1TokenType | undefined
  slpv2TokenType: Slpv2TokenType | undefined
  burnError: string
  actualBurn: string
  slpv2IntentionalBurn: string
  burnMintBatons: boolean
}

export interface Slpv1GenesisInfo {
  tokenTicker: string
  tokenName: string
  tokenDocumentUrl: string
  tokenDocumentHash: string
  decimals: number
}

export interface Slpv2GenesisInfo {
  tokenTicker: string
  tokenName: string
  url: string
  data: Uint8Array
  authPubkey: string
  decimals: number
}

export interface SlpToken {
  tokenId: string
  tokenProtocol: TokenProtocol
  slpv1TokenType: Slpv1TokenType | undefined
  slpv2TokenType: Slpv2TokenType | undefined
  slpv2SectionIdx: number
  isBurned: boolean
  amount: string
  isMintBaton: boolean
}

/** Input of a tx, spends an output of a previous tx. */
export interface TxInput {
  /** Points to an output spent by this input. */
  prevOut: OutPoint
  /** Script unlocking the output, in hex encoding.
   * Aka. `scriptSig` in bitcoind parlance. */
  inputScript: string
  /** Script of the output, in hex encoding.
   * Aka. `scriptPubKey` in bitcoind parlance. */
  outputScript: string | undefined
  /** Value of the output spent by this input, in satoshis. */
  value: string
  /** `sequence` field of the input; can be used for relative time locking. */
  sequenceNo: number
  /** SLP/SLPv2 tokens of this input, or `undefined` if no tokens. */
  slp: SlpToken | undefined
}

/** Output of a tx, creates new UTXOs. */
export interface TxOutput {
  /** Value of the output, in satoshis. */
  value: string
  /** Script of this output, locking the coins.
   * Aka. `scriptPubKey` in bitcoind parlance. */
  outputScript: string
  /** SLP/SLPv2 tokens locked up in this output, or `undefined` if no tokens
   * were sent to this output. */
  slp: SlpToken | undefined
  /** Transaction & input index spending this output, or undefined if
   * unspent. */
  spentBy: SpentBy | undefined
}

/** Metadata of a block, used in transaction data. */
export interface BlockMetadata {
  /** Height of the block. */
  height: number
  /** Hash of the block. */
  hash: string
  /** Timestamp of the block; useful if `timeFirstSeen` of a transaction is
   * unknown. */
  timestamp: string
  isFinal: boolean
}

/** Outpoint referencing an output on the blockchain (or input for field
 * `spentBy`). */
export interface OutPoint {
  /** Transaction referenced by this outpoint. */
  txid: string
  /** Index of the output in the tx referenced by this outpoint
   * (or input index if used in field `spentBy`). */
  outIdx: number
}

export interface SpentBy {
  txid: string
  inputIdx: number
}

/** SLP amount or whether this is a mint baton, for inputs and outputs. */
export interface SlpToken {
  /** SLP amount of the input or output, in base units. */
  amount: string
  /** Whether this input/output is a mint baton. */
  isMintBaton: boolean
}

/** Message returned from the WebSocket. */
export type WsMsg = Error | MsgTx | MsgBlock

/** A transaction has been added to the mempool. */
export interface MsgTx {
  type: "MsgTx"
  txMsgType: "AddedToMempool" | "RemovedFromMempool" | "Confirmed" | "Finalized"
  txid: string
}

/** A new block has been added to the chain. Sent regardless of subscriptions. */
export interface MsgBlock {
  type: "MsgBlock"
  blockMsgType: "Connected" | "Disconnected" | "Finalized"
  /** block hash of the block, in 'human-readable' (big-endian) hex encoding. */
  blockHash: string
  blockHeight: number
}

/** Reports an error, e.g. when a subscription is malformed. */
export interface Error {
  type: "Error"
  /** Human-readable message for this error. */
  msg: string
}

/** Script type queried in the `script` method.
 * - `other`: Script type not covered by the standard script types; payload is
 *   the raw hex.
 * - `p2pk`: Pay-to-Public-Key (`<pk> OP_CHECKSIG`), payload is the hex of the
 *   pubkey (compressed (33 bytes) or uncompressed (65 bytes)).
 * - `p2pkh`: Pay-to-Public-Key-Hash
 *   (`OP_DUP OP_HASH160 <pkh> OP_EQUALVERIFY OP_CHECKSIG`).
 *   Payload is the 20 byte public key hash.
 * - `p2sh`: Pay-to-Script-Hash (`OP_HASH160 <sh> OP_EQUAL`).
 *   Payload is the 20 byte script hash.
 */
export type ScriptType = "other" | "p2pk" | "p2pkh" | "p2sh"
