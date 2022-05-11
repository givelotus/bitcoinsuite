/* eslint-disable */
import Long from "long"
import * as _m0 from "protobufjs/minimal"

export const protobufPackage = "chronik"

export enum SlpTokenType {
  FUNGIBLE = 0,
  NFT1_GROUP = 1,
  NFT1_CHILD = 2,
  UNKNOWN_TOKEN_TYPE = 3,
  UNRECOGNIZED = -1,
}

export function slpTokenTypeFromJSON(object: any): SlpTokenType {
  switch (object) {
    case 0:
    case "FUNGIBLE":
      return SlpTokenType.FUNGIBLE
    case 1:
    case "NFT1_GROUP":
      return SlpTokenType.NFT1_GROUP
    case 2:
    case "NFT1_CHILD":
      return SlpTokenType.NFT1_CHILD
    case 3:
    case "UNKNOWN_TOKEN_TYPE":
      return SlpTokenType.UNKNOWN_TOKEN_TYPE
    case -1:
    case "UNRECOGNIZED":
    default:
      return SlpTokenType.UNRECOGNIZED
  }
}

export function slpTokenTypeToJSON(object: SlpTokenType): string {
  switch (object) {
    case SlpTokenType.FUNGIBLE:
      return "FUNGIBLE"
    case SlpTokenType.NFT1_GROUP:
      return "NFT1_GROUP"
    case SlpTokenType.NFT1_CHILD:
      return "NFT1_CHILD"
    case SlpTokenType.UNKNOWN_TOKEN_TYPE:
      return "UNKNOWN_TOKEN_TYPE"
    default:
      return "UNKNOWN"
  }
}

export enum SlpTxType {
  GENESIS = 0,
  SEND = 1,
  MINT = 2,
  UNKNOWN_TX_TYPE = 3,
  UNRECOGNIZED = -1,
}

export function slpTxTypeFromJSON(object: any): SlpTxType {
  switch (object) {
    case 0:
    case "GENESIS":
      return SlpTxType.GENESIS
    case 1:
    case "SEND":
      return SlpTxType.SEND
    case 2:
    case "MINT":
      return SlpTxType.MINT
    case 3:
    case "UNKNOWN_TX_TYPE":
      return SlpTxType.UNKNOWN_TX_TYPE
    case -1:
    case "UNRECOGNIZED":
    default:
      return SlpTxType.UNRECOGNIZED
  }
}

export function slpTxTypeToJSON(object: SlpTxType): string {
  switch (object) {
    case SlpTxType.GENESIS:
      return "GENESIS"
    case SlpTxType.SEND:
      return "SEND"
    case SlpTxType.MINT:
      return "MINT"
    case SlpTxType.UNKNOWN_TX_TYPE:
      return "UNKNOWN_TX_TYPE"
    default:
      return "UNKNOWN"
  }
}

export enum Network {
  BCH = 0,
  XEC = 1,
  XPI = 2,
  XRG = 3,
  UNRECOGNIZED = -1,
}

export function networkFromJSON(object: any): Network {
  switch (object) {
    case 0:
    case "BCH":
      return Network.BCH
    case 1:
    case "XEC":
      return Network.XEC
    case 2:
    case "XPI":
      return Network.XPI
    case 3:
    case "XRG":
      return Network.XRG
    case -1:
    case "UNRECOGNIZED":
    default:
      return Network.UNRECOGNIZED
  }
}

export function networkToJSON(object: Network): string {
  switch (object) {
    case Network.BCH:
      return "BCH"
    case Network.XEC:
      return "XEC"
    case Network.XPI:
      return "XPI"
    case Network.XRG:
      return "XRG"
    default:
      return "UNKNOWN"
  }
}

export enum UtxoStateVariant {
  UNSPENT = 0,
  SPENT = 1,
  NO_SUCH_TX = 2,
  NO_SUCH_OUTPUT = 3,
  UNRECOGNIZED = -1,
}

export function utxoStateVariantFromJSON(object: any): UtxoStateVariant {
  switch (object) {
    case 0:
    case "UNSPENT":
      return UtxoStateVariant.UNSPENT
    case 1:
    case "SPENT":
      return UtxoStateVariant.SPENT
    case 2:
    case "NO_SUCH_TX":
      return UtxoStateVariant.NO_SUCH_TX
    case 3:
    case "NO_SUCH_OUTPUT":
      return UtxoStateVariant.NO_SUCH_OUTPUT
    case -1:
    case "UNRECOGNIZED":
    default:
      return UtxoStateVariant.UNRECOGNIZED
  }
}

export function utxoStateVariantToJSON(object: UtxoStateVariant): string {
  switch (object) {
    case UtxoStateVariant.UNSPENT:
      return "UNSPENT"
    case UtxoStateVariant.SPENT:
      return "SPENT"
    case UtxoStateVariant.NO_SUCH_TX:
      return "NO_SUCH_TX"
    case UtxoStateVariant.NO_SUCH_OUTPUT:
      return "NO_SUCH_OUTPUT"
    default:
      return "UNKNOWN"
  }
}

export interface ValidateUtxoRequest {
  outpoints: OutPoint[]
}

export interface ValidateUtxoResponse {
  utxoStates: UtxoState[]
}

export interface BroadcastTxRequest {
  rawTx: Uint8Array
  skipSlpCheck: boolean
}

export interface BroadcastTxResponse {
  txid: Uint8Array
}

export interface BroadcastTxsRequest {
  rawTxs: Uint8Array[]
  skipSlpCheck: boolean
}

export interface BroadcastTxsResponse {
  txids: Uint8Array[]
}

export interface BlockchainInfo {
  tipHash: Uint8Array
  tipHeight: number
}

export interface Tx {
  txid: Uint8Array
  version: number
  inputs: TxInput[]
  outputs: TxOutput[]
  lockTime: number
  slpTxData: SlpTxData | undefined
  slpErrorMsg: string
  block: BlockMetadata | undefined
  timeFirstSeen: Long
  network: Network
}

export interface Utxo {
  outpoint: OutPoint | undefined
  blockHeight: number
  isCoinbase: boolean
  value: Long
  slpMeta: SlpMeta | undefined
  slpToken: SlpToken | undefined
  network: Network
}

export interface Token {
  slpTxData: SlpTxData | undefined
  tokenStats: TokenStats | undefined
}

export interface BlockInfo {
  hash: Uint8Array
  prevHash: Uint8Array
  height: number
  nBits: number
  timestamp: Long
  /** Block size of this block in bytes (including headers etc.) */
  blockSize: Long
  /** Number of txs in this block */
  numTxs: Long
  /** Total number of tx inputs in block (including coinbase) */
  numInputs: Long
  /** Total number of tx output in block (including coinbase) */
  numOutputs: Long
  /** Total number of satoshis spent by tx inputs */
  sumInputSats: Long
  /** Block reward for this block */
  sumCoinbaseOutputSats: Long
  /** Total number of satoshis in non-coinbase tx outputs */
  sumNormalOutputSats: Long
  /** Total number of satoshis burned using OP_RETURN */
  sumBurnedSats: Long
}

export interface BlockDetails {
  version: number
  merkleRoot: Uint8Array
  nonce: Long
  medianTimestamp: Long
}

export interface Block {
  blockInfo: BlockInfo | undefined
  blockDetails: BlockDetails | undefined
  txs: Tx[]
}

export interface ScriptUtxos {
  outputScript: Uint8Array
  utxos: Utxo[]
}

export interface TxHistoryPage {
  txs: Tx[]
  numPages: number
}

export interface Utxos {
  scriptUtxos: ScriptUtxos[]
}

export interface Blocks {
  blocks: BlockInfo[]
}

export interface SlpTxData {
  slpMeta: SlpMeta | undefined
  genesisInfo: SlpGenesisInfo | undefined
}

export interface SlpMeta {
  tokenType: SlpTokenType
  txType: SlpTxType
  tokenId: Uint8Array
  groupTokenId: Uint8Array
}

export interface TokenStats {
  /**
   * This doesn't fit into uint64, so we use a string with the decimal
   * representation. If available, use i128 to parse, otherwise some
   * BigNumber library.
   */
  totalMinted: string
  totalBurned: string
}

export interface TxInput {
  prevOut: OutPoint | undefined
  inputScript: Uint8Array
  outputScript: Uint8Array
  value: Long
  sequenceNo: number
  slpBurn: SlpBurn | undefined
  slpToken: SlpToken | undefined
}

export interface TxOutput {
  value: Long
  outputScript: Uint8Array
  slpToken: SlpToken | undefined
  spentBy: OutPoint | undefined
}

export interface BlockMetadata {
  height: number
  hash: Uint8Array
  timestamp: Long
}

export interface OutPoint {
  txid: Uint8Array
  outIdx: number
}

export interface SlpToken {
  amount: Long
  isMintBaton: boolean
}

export interface SlpBurn {
  token: SlpToken | undefined
  tokenId: Uint8Array
}

export interface SlpGenesisInfo {
  tokenTicker: Uint8Array
  tokenName: Uint8Array
  tokenDocumentUrl: Uint8Array
  tokenDocumentHash: Uint8Array
  decimals: number
}

export interface UtxoState {
  height: number
  isConfirmed: boolean
  state: UtxoStateVariant
}

export interface Subscription {
  scriptType: string
  payload: Uint8Array
  isSubscribe: boolean
}

export interface SubscribeMsg {
  error: Error | undefined
  AddedToMempool: MsgAddedToMempool | undefined
  RemovedFromMempool: MsgRemovedFromMempool | undefined
  Confirmed: MsgConfirmed | undefined
  Reorg: MsgReorg | undefined
  BlockConnected: MsgBlockConnected | undefined
  BlockDisconnected: MsgBlockDisconnected | undefined
}

export interface MsgAddedToMempool {
  txid: Uint8Array
}

export interface MsgRemovedFromMempool {
  txid: Uint8Array
}

export interface MsgConfirmed {
  txid: Uint8Array
}

export interface MsgReorg {
  txid: Uint8Array
}

export interface MsgBlockConnected {
  blockHash: Uint8Array
}

export interface MsgBlockDisconnected {
  blockHash: Uint8Array
}

export interface Error {
  errorCode: string
  msg: string
  isUserError: boolean
}

function createBaseValidateUtxoRequest(): ValidateUtxoRequest {
  return { outpoints: [] }
}

export const ValidateUtxoRequest = {
  encode(
    message: ValidateUtxoRequest,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    for (const v of message.outpoints) {
      OutPoint.encode(v!, writer.uint32(10).fork()).ldelim()
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ValidateUtxoRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseValidateUtxoRequest()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.outpoints.push(OutPoint.decode(reader, reader.uint32()))
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): ValidateUtxoRequest {
    return {
      outpoints: Array.isArray(object?.outpoints)
        ? object.outpoints.map((e: any) => OutPoint.fromJSON(e))
        : [],
    }
  },

  toJSON(message: ValidateUtxoRequest): unknown {
    const obj: any = {}
    if (message.outpoints) {
      obj.outpoints = message.outpoints.map(e =>
        e ? OutPoint.toJSON(e) : undefined,
      )
    } else {
      obj.outpoints = []
    }
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<ValidateUtxoRequest>, I>>(
    object: I,
  ): ValidateUtxoRequest {
    const message = createBaseValidateUtxoRequest()
    message.outpoints =
      object.outpoints?.map(e => OutPoint.fromPartial(e)) || []
    return message
  },
}

function createBaseValidateUtxoResponse(): ValidateUtxoResponse {
  return { utxoStates: [] }
}

export const ValidateUtxoResponse = {
  encode(
    message: ValidateUtxoResponse,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    for (const v of message.utxoStates) {
      UtxoState.encode(v!, writer.uint32(10).fork()).ldelim()
    }
    return writer
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number,
  ): ValidateUtxoResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseValidateUtxoResponse()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.utxoStates.push(UtxoState.decode(reader, reader.uint32()))
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): ValidateUtxoResponse {
    return {
      utxoStates: Array.isArray(object?.utxoStates)
        ? object.utxoStates.map((e: any) => UtxoState.fromJSON(e))
        : [],
    }
  },

  toJSON(message: ValidateUtxoResponse): unknown {
    const obj: any = {}
    if (message.utxoStates) {
      obj.utxoStates = message.utxoStates.map(e =>
        e ? UtxoState.toJSON(e) : undefined,
      )
    } else {
      obj.utxoStates = []
    }
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<ValidateUtxoResponse>, I>>(
    object: I,
  ): ValidateUtxoResponse {
    const message = createBaseValidateUtxoResponse()
    message.utxoStates =
      object.utxoStates?.map(e => UtxoState.fromPartial(e)) || []
    return message
  },
}

function createBaseBroadcastTxRequest(): BroadcastTxRequest {
  return { rawTx: new Uint8Array(), skipSlpCheck: false }
}

export const BroadcastTxRequest = {
  encode(
    message: BroadcastTxRequest,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.rawTx.length !== 0) {
      writer.uint32(10).bytes(message.rawTx)
    }
    if (message.skipSlpCheck === true) {
      writer.uint32(16).bool(message.skipSlpCheck)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): BroadcastTxRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseBroadcastTxRequest()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.rawTx = reader.bytes()
          break
        case 2:
          message.skipSlpCheck = reader.bool()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): BroadcastTxRequest {
    return {
      rawTx: isSet(object.rawTx)
        ? bytesFromBase64(object.rawTx)
        : new Uint8Array(),
      skipSlpCheck: isSet(object.skipSlpCheck)
        ? Boolean(object.skipSlpCheck)
        : false,
    }
  },

  toJSON(message: BroadcastTxRequest): unknown {
    const obj: any = {}
    message.rawTx !== undefined &&
      (obj.rawTx = base64FromBytes(
        message.rawTx !== undefined ? message.rawTx : new Uint8Array(),
      ))
    message.skipSlpCheck !== undefined &&
      (obj.skipSlpCheck = message.skipSlpCheck)
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<BroadcastTxRequest>, I>>(
    object: I,
  ): BroadcastTxRequest {
    const message = createBaseBroadcastTxRequest()
    message.rawTx = object.rawTx ?? new Uint8Array()
    message.skipSlpCheck = object.skipSlpCheck ?? false
    return message
  },
}

function createBaseBroadcastTxResponse(): BroadcastTxResponse {
  return { txid: new Uint8Array() }
}

export const BroadcastTxResponse = {
  encode(
    message: BroadcastTxResponse,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.txid.length !== 0) {
      writer.uint32(10).bytes(message.txid)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): BroadcastTxResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseBroadcastTxResponse()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.txid = reader.bytes()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): BroadcastTxResponse {
    return {
      txid: isSet(object.txid)
        ? bytesFromBase64(object.txid)
        : new Uint8Array(),
    }
  },

  toJSON(message: BroadcastTxResponse): unknown {
    const obj: any = {}
    message.txid !== undefined &&
      (obj.txid = base64FromBytes(
        message.txid !== undefined ? message.txid : new Uint8Array(),
      ))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<BroadcastTxResponse>, I>>(
    object: I,
  ): BroadcastTxResponse {
    const message = createBaseBroadcastTxResponse()
    message.txid = object.txid ?? new Uint8Array()
    return message
  },
}

function createBaseBroadcastTxsRequest(): BroadcastTxsRequest {
  return { rawTxs: [], skipSlpCheck: false }
}

export const BroadcastTxsRequest = {
  encode(
    message: BroadcastTxsRequest,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    for (const v of message.rawTxs) {
      writer.uint32(10).bytes(v!)
    }
    if (message.skipSlpCheck === true) {
      writer.uint32(16).bool(message.skipSlpCheck)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): BroadcastTxsRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseBroadcastTxsRequest()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.rawTxs.push(reader.bytes())
          break
        case 2:
          message.skipSlpCheck = reader.bool()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): BroadcastTxsRequest {
    return {
      rawTxs: Array.isArray(object?.rawTxs)
        ? object.rawTxs.map((e: any) => bytesFromBase64(e))
        : [],
      skipSlpCheck: isSet(object.skipSlpCheck)
        ? Boolean(object.skipSlpCheck)
        : false,
    }
  },

  toJSON(message: BroadcastTxsRequest): unknown {
    const obj: any = {}
    if (message.rawTxs) {
      obj.rawTxs = message.rawTxs.map(e =>
        base64FromBytes(e !== undefined ? e : new Uint8Array()),
      )
    } else {
      obj.rawTxs = []
    }
    message.skipSlpCheck !== undefined &&
      (obj.skipSlpCheck = message.skipSlpCheck)
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<BroadcastTxsRequest>, I>>(
    object: I,
  ): BroadcastTxsRequest {
    const message = createBaseBroadcastTxsRequest()
    message.rawTxs = object.rawTxs?.map(e => e) || []
    message.skipSlpCheck = object.skipSlpCheck ?? false
    return message
  },
}

function createBaseBroadcastTxsResponse(): BroadcastTxsResponse {
  return { txids: [] }
}

export const BroadcastTxsResponse = {
  encode(
    message: BroadcastTxsResponse,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    for (const v of message.txids) {
      writer.uint32(10).bytes(v!)
    }
    return writer
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number,
  ): BroadcastTxsResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseBroadcastTxsResponse()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.txids.push(reader.bytes())
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): BroadcastTxsResponse {
    return {
      txids: Array.isArray(object?.txids)
        ? object.txids.map((e: any) => bytesFromBase64(e))
        : [],
    }
  },

  toJSON(message: BroadcastTxsResponse): unknown {
    const obj: any = {}
    if (message.txids) {
      obj.txids = message.txids.map(e =>
        base64FromBytes(e !== undefined ? e : new Uint8Array()),
      )
    } else {
      obj.txids = []
    }
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<BroadcastTxsResponse>, I>>(
    object: I,
  ): BroadcastTxsResponse {
    const message = createBaseBroadcastTxsResponse()
    message.txids = object.txids?.map(e => e) || []
    return message
  },
}

function createBaseBlockchainInfo(): BlockchainInfo {
  return { tipHash: new Uint8Array(), tipHeight: 0 }
}

export const BlockchainInfo = {
  encode(
    message: BlockchainInfo,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.tipHash.length !== 0) {
      writer.uint32(10).bytes(message.tipHash)
    }
    if (message.tipHeight !== 0) {
      writer.uint32(16).int32(message.tipHeight)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): BlockchainInfo {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseBlockchainInfo()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.tipHash = reader.bytes()
          break
        case 2:
          message.tipHeight = reader.int32()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): BlockchainInfo {
    return {
      tipHash: isSet(object.tipHash)
        ? bytesFromBase64(object.tipHash)
        : new Uint8Array(),
      tipHeight: isSet(object.tipHeight) ? Number(object.tipHeight) : 0,
    }
  },

  toJSON(message: BlockchainInfo): unknown {
    const obj: any = {}
    message.tipHash !== undefined &&
      (obj.tipHash = base64FromBytes(
        message.tipHash !== undefined ? message.tipHash : new Uint8Array(),
      ))
    message.tipHeight !== undefined &&
      (obj.tipHeight = Math.round(message.tipHeight))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<BlockchainInfo>, I>>(
    object: I,
  ): BlockchainInfo {
    const message = createBaseBlockchainInfo()
    message.tipHash = object.tipHash ?? new Uint8Array()
    message.tipHeight = object.tipHeight ?? 0
    return message
  },
}

function createBaseTx(): Tx {
  return {
    txid: new Uint8Array(),
    version: 0,
    inputs: [],
    outputs: [],
    lockTime: 0,
    slpTxData: undefined,
    slpErrorMsg: "",
    block: undefined,
    timeFirstSeen: Long.ZERO,
    network: 0,
  }
}

export const Tx = {
  encode(message: Tx, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.txid.length !== 0) {
      writer.uint32(10).bytes(message.txid)
    }
    if (message.version !== 0) {
      writer.uint32(16).int32(message.version)
    }
    for (const v of message.inputs) {
      TxInput.encode(v!, writer.uint32(26).fork()).ldelim()
    }
    for (const v of message.outputs) {
      TxOutput.encode(v!, writer.uint32(34).fork()).ldelim()
    }
    if (message.lockTime !== 0) {
      writer.uint32(40).uint32(message.lockTime)
    }
    if (message.slpTxData !== undefined) {
      SlpTxData.encode(message.slpTxData, writer.uint32(50).fork()).ldelim()
    }
    if (message.slpErrorMsg !== "") {
      writer.uint32(58).string(message.slpErrorMsg)
    }
    if (message.block !== undefined) {
      BlockMetadata.encode(message.block, writer.uint32(66).fork()).ldelim()
    }
    if (!message.timeFirstSeen.isZero()) {
      writer.uint32(72).int64(message.timeFirstSeen)
    }
    if (message.network !== 0) {
      writer.uint32(80).int32(message.network)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Tx {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseTx()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.txid = reader.bytes()
          break
        case 2:
          message.version = reader.int32()
          break
        case 3:
          message.inputs.push(TxInput.decode(reader, reader.uint32()))
          break
        case 4:
          message.outputs.push(TxOutput.decode(reader, reader.uint32()))
          break
        case 5:
          message.lockTime = reader.uint32()
          break
        case 6:
          message.slpTxData = SlpTxData.decode(reader, reader.uint32())
          break
        case 7:
          message.slpErrorMsg = reader.string()
          break
        case 8:
          message.block = BlockMetadata.decode(reader, reader.uint32())
          break
        case 9:
          message.timeFirstSeen = reader.int64() as Long
          break
        case 10:
          message.network = reader.int32() as any
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): Tx {
    return {
      txid: isSet(object.txid)
        ? bytesFromBase64(object.txid)
        : new Uint8Array(),
      version: isSet(object.version) ? Number(object.version) : 0,
      inputs: Array.isArray(object?.inputs)
        ? object.inputs.map((e: any) => TxInput.fromJSON(e))
        : [],
      outputs: Array.isArray(object?.outputs)
        ? object.outputs.map((e: any) => TxOutput.fromJSON(e))
        : [],
      lockTime: isSet(object.lockTime) ? Number(object.lockTime) : 0,
      slpTxData: isSet(object.slpTxData)
        ? SlpTxData.fromJSON(object.slpTxData)
        : undefined,
      slpErrorMsg: isSet(object.slpErrorMsg) ? String(object.slpErrorMsg) : "",
      block: isSet(object.block)
        ? BlockMetadata.fromJSON(object.block)
        : undefined,
      timeFirstSeen: isSet(object.timeFirstSeen)
        ? Long.fromValue(object.timeFirstSeen)
        : Long.ZERO,
      network: isSet(object.network) ? networkFromJSON(object.network) : 0,
    }
  },

  toJSON(message: Tx): unknown {
    const obj: any = {}
    message.txid !== undefined &&
      (obj.txid = base64FromBytes(
        message.txid !== undefined ? message.txid : new Uint8Array(),
      ))
    message.version !== undefined && (obj.version = Math.round(message.version))
    if (message.inputs) {
      obj.inputs = message.inputs.map(e => (e ? TxInput.toJSON(e) : undefined))
    } else {
      obj.inputs = []
    }
    if (message.outputs) {
      obj.outputs = message.outputs.map(e =>
        e ? TxOutput.toJSON(e) : undefined,
      )
    } else {
      obj.outputs = []
    }
    message.lockTime !== undefined &&
      (obj.lockTime = Math.round(message.lockTime))
    message.slpTxData !== undefined &&
      (obj.slpTxData = message.slpTxData
        ? SlpTxData.toJSON(message.slpTxData)
        : undefined)
    message.slpErrorMsg !== undefined && (obj.slpErrorMsg = message.slpErrorMsg)
    message.block !== undefined &&
      (obj.block = message.block
        ? BlockMetadata.toJSON(message.block)
        : undefined)
    message.timeFirstSeen !== undefined &&
      (obj.timeFirstSeen = (message.timeFirstSeen || Long.ZERO).toString())
    message.network !== undefined &&
      (obj.network = networkToJSON(message.network))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<Tx>, I>>(object: I): Tx {
    const message = createBaseTx()
    message.txid = object.txid ?? new Uint8Array()
    message.version = object.version ?? 0
    message.inputs = object.inputs?.map(e => TxInput.fromPartial(e)) || []
    message.outputs = object.outputs?.map(e => TxOutput.fromPartial(e)) || []
    message.lockTime = object.lockTime ?? 0
    message.slpTxData =
      object.slpTxData !== undefined && object.slpTxData !== null
        ? SlpTxData.fromPartial(object.slpTxData)
        : undefined
    message.slpErrorMsg = object.slpErrorMsg ?? ""
    message.block =
      object.block !== undefined && object.block !== null
        ? BlockMetadata.fromPartial(object.block)
        : undefined
    message.timeFirstSeen =
      object.timeFirstSeen !== undefined && object.timeFirstSeen !== null
        ? Long.fromValue(object.timeFirstSeen)
        : Long.ZERO
    message.network = object.network ?? 0
    return message
  },
}

function createBaseUtxo(): Utxo {
  return {
    outpoint: undefined,
    blockHeight: 0,
    isCoinbase: false,
    value: Long.ZERO,
    slpMeta: undefined,
    slpToken: undefined,
    network: 0,
  }
}

export const Utxo = {
  encode(message: Utxo, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.outpoint !== undefined) {
      OutPoint.encode(message.outpoint, writer.uint32(10).fork()).ldelim()
    }
    if (message.blockHeight !== 0) {
      writer.uint32(16).int32(message.blockHeight)
    }
    if (message.isCoinbase === true) {
      writer.uint32(24).bool(message.isCoinbase)
    }
    if (!message.value.isZero()) {
      writer.uint32(40).int64(message.value)
    }
    if (message.slpMeta !== undefined) {
      SlpMeta.encode(message.slpMeta, writer.uint32(50).fork()).ldelim()
    }
    if (message.slpToken !== undefined) {
      SlpToken.encode(message.slpToken, writer.uint32(58).fork()).ldelim()
    }
    if (message.network !== 0) {
      writer.uint32(72).int32(message.network)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Utxo {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseUtxo()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.outpoint = OutPoint.decode(reader, reader.uint32())
          break
        case 2:
          message.blockHeight = reader.int32()
          break
        case 3:
          message.isCoinbase = reader.bool()
          break
        case 5:
          message.value = reader.int64() as Long
          break
        case 6:
          message.slpMeta = SlpMeta.decode(reader, reader.uint32())
          break
        case 7:
          message.slpToken = SlpToken.decode(reader, reader.uint32())
          break
        case 9:
          message.network = reader.int32() as any
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): Utxo {
    return {
      outpoint: isSet(object.outpoint)
        ? OutPoint.fromJSON(object.outpoint)
        : undefined,
      blockHeight: isSet(object.blockHeight) ? Number(object.blockHeight) : 0,
      isCoinbase: isSet(object.isCoinbase) ? Boolean(object.isCoinbase) : false,
      value: isSet(object.value) ? Long.fromValue(object.value) : Long.ZERO,
      slpMeta: isSet(object.slpMeta)
        ? SlpMeta.fromJSON(object.slpMeta)
        : undefined,
      slpToken: isSet(object.slpToken)
        ? SlpToken.fromJSON(object.slpToken)
        : undefined,
      network: isSet(object.network) ? networkFromJSON(object.network) : 0,
    }
  },

  toJSON(message: Utxo): unknown {
    const obj: any = {}
    message.outpoint !== undefined &&
      (obj.outpoint = message.outpoint
        ? OutPoint.toJSON(message.outpoint)
        : undefined)
    message.blockHeight !== undefined &&
      (obj.blockHeight = Math.round(message.blockHeight))
    message.isCoinbase !== undefined && (obj.isCoinbase = message.isCoinbase)
    message.value !== undefined &&
      (obj.value = (message.value || Long.ZERO).toString())
    message.slpMeta !== undefined &&
      (obj.slpMeta = message.slpMeta
        ? SlpMeta.toJSON(message.slpMeta)
        : undefined)
    message.slpToken !== undefined &&
      (obj.slpToken = message.slpToken
        ? SlpToken.toJSON(message.slpToken)
        : undefined)
    message.network !== undefined &&
      (obj.network = networkToJSON(message.network))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<Utxo>, I>>(object: I): Utxo {
    const message = createBaseUtxo()
    message.outpoint =
      object.outpoint !== undefined && object.outpoint !== null
        ? OutPoint.fromPartial(object.outpoint)
        : undefined
    message.blockHeight = object.blockHeight ?? 0
    message.isCoinbase = object.isCoinbase ?? false
    message.value =
      object.value !== undefined && object.value !== null
        ? Long.fromValue(object.value)
        : Long.ZERO
    message.slpMeta =
      object.slpMeta !== undefined && object.slpMeta !== null
        ? SlpMeta.fromPartial(object.slpMeta)
        : undefined
    message.slpToken =
      object.slpToken !== undefined && object.slpToken !== null
        ? SlpToken.fromPartial(object.slpToken)
        : undefined
    message.network = object.network ?? 0
    return message
  },
}

function createBaseToken(): Token {
  return { slpTxData: undefined, tokenStats: undefined }
}

export const Token = {
  encode(message: Token, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.slpTxData !== undefined) {
      SlpTxData.encode(message.slpTxData, writer.uint32(10).fork()).ldelim()
    }
    if (message.tokenStats !== undefined) {
      TokenStats.encode(message.tokenStats, writer.uint32(18).fork()).ldelim()
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Token {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseToken()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.slpTxData = SlpTxData.decode(reader, reader.uint32())
          break
        case 2:
          message.tokenStats = TokenStats.decode(reader, reader.uint32())
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): Token {
    return {
      slpTxData: isSet(object.slpTxData)
        ? SlpTxData.fromJSON(object.slpTxData)
        : undefined,
      tokenStats: isSet(object.tokenStats)
        ? TokenStats.fromJSON(object.tokenStats)
        : undefined,
    }
  },

  toJSON(message: Token): unknown {
    const obj: any = {}
    message.slpTxData !== undefined &&
      (obj.slpTxData = message.slpTxData
        ? SlpTxData.toJSON(message.slpTxData)
        : undefined)
    message.tokenStats !== undefined &&
      (obj.tokenStats = message.tokenStats
        ? TokenStats.toJSON(message.tokenStats)
        : undefined)
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<Token>, I>>(object: I): Token {
    const message = createBaseToken()
    message.slpTxData =
      object.slpTxData !== undefined && object.slpTxData !== null
        ? SlpTxData.fromPartial(object.slpTxData)
        : undefined
    message.tokenStats =
      object.tokenStats !== undefined && object.tokenStats !== null
        ? TokenStats.fromPartial(object.tokenStats)
        : undefined
    return message
  },
}

function createBaseBlockInfo(): BlockInfo {
  return {
    hash: new Uint8Array(),
    prevHash: new Uint8Array(),
    height: 0,
    nBits: 0,
    timestamp: Long.ZERO,
    blockSize: Long.UZERO,
    numTxs: Long.UZERO,
    numInputs: Long.UZERO,
    numOutputs: Long.UZERO,
    sumInputSats: Long.ZERO,
    sumCoinbaseOutputSats: Long.ZERO,
    sumNormalOutputSats: Long.ZERO,
    sumBurnedSats: Long.ZERO,
  }
}

export const BlockInfo = {
  encode(
    message: BlockInfo,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.hash.length !== 0) {
      writer.uint32(10).bytes(message.hash)
    }
    if (message.prevHash.length !== 0) {
      writer.uint32(18).bytes(message.prevHash)
    }
    if (message.height !== 0) {
      writer.uint32(24).int32(message.height)
    }
    if (message.nBits !== 0) {
      writer.uint32(32).uint32(message.nBits)
    }
    if (!message.timestamp.isZero()) {
      writer.uint32(40).int64(message.timestamp)
    }
    if (!message.blockSize.isZero()) {
      writer.uint32(48).uint64(message.blockSize)
    }
    if (!message.numTxs.isZero()) {
      writer.uint32(56).uint64(message.numTxs)
    }
    if (!message.numInputs.isZero()) {
      writer.uint32(64).uint64(message.numInputs)
    }
    if (!message.numOutputs.isZero()) {
      writer.uint32(72).uint64(message.numOutputs)
    }
    if (!message.sumInputSats.isZero()) {
      writer.uint32(80).int64(message.sumInputSats)
    }
    if (!message.sumCoinbaseOutputSats.isZero()) {
      writer.uint32(88).int64(message.sumCoinbaseOutputSats)
    }
    if (!message.sumNormalOutputSats.isZero()) {
      writer.uint32(96).int64(message.sumNormalOutputSats)
    }
    if (!message.sumBurnedSats.isZero()) {
      writer.uint32(104).int64(message.sumBurnedSats)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): BlockInfo {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseBlockInfo()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.hash = reader.bytes()
          break
        case 2:
          message.prevHash = reader.bytes()
          break
        case 3:
          message.height = reader.int32()
          break
        case 4:
          message.nBits = reader.uint32()
          break
        case 5:
          message.timestamp = reader.int64() as Long
          break
        case 6:
          message.blockSize = reader.uint64() as Long
          break
        case 7:
          message.numTxs = reader.uint64() as Long
          break
        case 8:
          message.numInputs = reader.uint64() as Long
          break
        case 9:
          message.numOutputs = reader.uint64() as Long
          break
        case 10:
          message.sumInputSats = reader.int64() as Long
          break
        case 11:
          message.sumCoinbaseOutputSats = reader.int64() as Long
          break
        case 12:
          message.sumNormalOutputSats = reader.int64() as Long
          break
        case 13:
          message.sumBurnedSats = reader.int64() as Long
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): BlockInfo {
    return {
      hash: isSet(object.hash)
        ? bytesFromBase64(object.hash)
        : new Uint8Array(),
      prevHash: isSet(object.prevHash)
        ? bytesFromBase64(object.prevHash)
        : new Uint8Array(),
      height: isSet(object.height) ? Number(object.height) : 0,
      nBits: isSet(object.nBits) ? Number(object.nBits) : 0,
      timestamp: isSet(object.timestamp)
        ? Long.fromValue(object.timestamp)
        : Long.ZERO,
      blockSize: isSet(object.blockSize)
        ? Long.fromValue(object.blockSize)
        : Long.UZERO,
      numTxs: isSet(object.numTxs) ? Long.fromValue(object.numTxs) : Long.UZERO,
      numInputs: isSet(object.numInputs)
        ? Long.fromValue(object.numInputs)
        : Long.UZERO,
      numOutputs: isSet(object.numOutputs)
        ? Long.fromValue(object.numOutputs)
        : Long.UZERO,
      sumInputSats: isSet(object.sumInputSats)
        ? Long.fromValue(object.sumInputSats)
        : Long.ZERO,
      sumCoinbaseOutputSats: isSet(object.sumCoinbaseOutputSats)
        ? Long.fromValue(object.sumCoinbaseOutputSats)
        : Long.ZERO,
      sumNormalOutputSats: isSet(object.sumNormalOutputSats)
        ? Long.fromValue(object.sumNormalOutputSats)
        : Long.ZERO,
      sumBurnedSats: isSet(object.sumBurnedSats)
        ? Long.fromValue(object.sumBurnedSats)
        : Long.ZERO,
    }
  },

  toJSON(message: BlockInfo): unknown {
    const obj: any = {}
    message.hash !== undefined &&
      (obj.hash = base64FromBytes(
        message.hash !== undefined ? message.hash : new Uint8Array(),
      ))
    message.prevHash !== undefined &&
      (obj.prevHash = base64FromBytes(
        message.prevHash !== undefined ? message.prevHash : new Uint8Array(),
      ))
    message.height !== undefined && (obj.height = Math.round(message.height))
    message.nBits !== undefined && (obj.nBits = Math.round(message.nBits))
    message.timestamp !== undefined &&
      (obj.timestamp = (message.timestamp || Long.ZERO).toString())
    message.blockSize !== undefined &&
      (obj.blockSize = (message.blockSize || Long.UZERO).toString())
    message.numTxs !== undefined &&
      (obj.numTxs = (message.numTxs || Long.UZERO).toString())
    message.numInputs !== undefined &&
      (obj.numInputs = (message.numInputs || Long.UZERO).toString())
    message.numOutputs !== undefined &&
      (obj.numOutputs = (message.numOutputs || Long.UZERO).toString())
    message.sumInputSats !== undefined &&
      (obj.sumInputSats = (message.sumInputSats || Long.ZERO).toString())
    message.sumCoinbaseOutputSats !== undefined &&
      (obj.sumCoinbaseOutputSats = (
        message.sumCoinbaseOutputSats || Long.ZERO
      ).toString())
    message.sumNormalOutputSats !== undefined &&
      (obj.sumNormalOutputSats = (
        message.sumNormalOutputSats || Long.ZERO
      ).toString())
    message.sumBurnedSats !== undefined &&
      (obj.sumBurnedSats = (message.sumBurnedSats || Long.ZERO).toString())
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<BlockInfo>, I>>(
    object: I,
  ): BlockInfo {
    const message = createBaseBlockInfo()
    message.hash = object.hash ?? new Uint8Array()
    message.prevHash = object.prevHash ?? new Uint8Array()
    message.height = object.height ?? 0
    message.nBits = object.nBits ?? 0
    message.timestamp =
      object.timestamp !== undefined && object.timestamp !== null
        ? Long.fromValue(object.timestamp)
        : Long.ZERO
    message.blockSize =
      object.blockSize !== undefined && object.blockSize !== null
        ? Long.fromValue(object.blockSize)
        : Long.UZERO
    message.numTxs =
      object.numTxs !== undefined && object.numTxs !== null
        ? Long.fromValue(object.numTxs)
        : Long.UZERO
    message.numInputs =
      object.numInputs !== undefined && object.numInputs !== null
        ? Long.fromValue(object.numInputs)
        : Long.UZERO
    message.numOutputs =
      object.numOutputs !== undefined && object.numOutputs !== null
        ? Long.fromValue(object.numOutputs)
        : Long.UZERO
    message.sumInputSats =
      object.sumInputSats !== undefined && object.sumInputSats !== null
        ? Long.fromValue(object.sumInputSats)
        : Long.ZERO
    message.sumCoinbaseOutputSats =
      object.sumCoinbaseOutputSats !== undefined &&
      object.sumCoinbaseOutputSats !== null
        ? Long.fromValue(object.sumCoinbaseOutputSats)
        : Long.ZERO
    message.sumNormalOutputSats =
      object.sumNormalOutputSats !== undefined &&
      object.sumNormalOutputSats !== null
        ? Long.fromValue(object.sumNormalOutputSats)
        : Long.ZERO
    message.sumBurnedSats =
      object.sumBurnedSats !== undefined && object.sumBurnedSats !== null
        ? Long.fromValue(object.sumBurnedSats)
        : Long.ZERO
    return message
  },
}

function createBaseBlockDetails(): BlockDetails {
  return {
    version: 0,
    merkleRoot: new Uint8Array(),
    nonce: Long.UZERO,
    medianTimestamp: Long.ZERO,
  }
}

export const BlockDetails = {
  encode(
    message: BlockDetails,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.version !== 0) {
      writer.uint32(8).int32(message.version)
    }
    if (message.merkleRoot.length !== 0) {
      writer.uint32(18).bytes(message.merkleRoot)
    }
    if (!message.nonce.isZero()) {
      writer.uint32(24).uint64(message.nonce)
    }
    if (!message.medianTimestamp.isZero()) {
      writer.uint32(32).int64(message.medianTimestamp)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): BlockDetails {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseBlockDetails()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.version = reader.int32()
          break
        case 2:
          message.merkleRoot = reader.bytes()
          break
        case 3:
          message.nonce = reader.uint64() as Long
          break
        case 4:
          message.medianTimestamp = reader.int64() as Long
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): BlockDetails {
    return {
      version: isSet(object.version) ? Number(object.version) : 0,
      merkleRoot: isSet(object.merkleRoot)
        ? bytesFromBase64(object.merkleRoot)
        : new Uint8Array(),
      nonce: isSet(object.nonce) ? Long.fromValue(object.nonce) : Long.UZERO,
      medianTimestamp: isSet(object.medianTimestamp)
        ? Long.fromValue(object.medianTimestamp)
        : Long.ZERO,
    }
  },

  toJSON(message: BlockDetails): unknown {
    const obj: any = {}
    message.version !== undefined && (obj.version = Math.round(message.version))
    message.merkleRoot !== undefined &&
      (obj.merkleRoot = base64FromBytes(
        message.merkleRoot !== undefined
          ? message.merkleRoot
          : new Uint8Array(),
      ))
    message.nonce !== undefined &&
      (obj.nonce = (message.nonce || Long.UZERO).toString())
    message.medianTimestamp !== undefined &&
      (obj.medianTimestamp = (message.medianTimestamp || Long.ZERO).toString())
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<BlockDetails>, I>>(
    object: I,
  ): BlockDetails {
    const message = createBaseBlockDetails()
    message.version = object.version ?? 0
    message.merkleRoot = object.merkleRoot ?? new Uint8Array()
    message.nonce =
      object.nonce !== undefined && object.nonce !== null
        ? Long.fromValue(object.nonce)
        : Long.UZERO
    message.medianTimestamp =
      object.medianTimestamp !== undefined && object.medianTimestamp !== null
        ? Long.fromValue(object.medianTimestamp)
        : Long.ZERO
    return message
  },
}

function createBaseBlock(): Block {
  return { blockInfo: undefined, blockDetails: undefined, txs: [] }
}

export const Block = {
  encode(message: Block, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.blockInfo !== undefined) {
      BlockInfo.encode(message.blockInfo, writer.uint32(10).fork()).ldelim()
    }
    if (message.blockDetails !== undefined) {
      BlockDetails.encode(
        message.blockDetails,
        writer.uint32(26).fork(),
      ).ldelim()
    }
    for (const v of message.txs) {
      Tx.encode(v!, writer.uint32(18).fork()).ldelim()
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Block {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseBlock()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.blockInfo = BlockInfo.decode(reader, reader.uint32())
          break
        case 3:
          message.blockDetails = BlockDetails.decode(reader, reader.uint32())
          break
        case 2:
          message.txs.push(Tx.decode(reader, reader.uint32()))
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): Block {
    return {
      blockInfo: isSet(object.blockInfo)
        ? BlockInfo.fromJSON(object.blockInfo)
        : undefined,
      blockDetails: isSet(object.blockDetails)
        ? BlockDetails.fromJSON(object.blockDetails)
        : undefined,
      txs: Array.isArray(object?.txs)
        ? object.txs.map((e: any) => Tx.fromJSON(e))
        : [],
    }
  },

  toJSON(message: Block): unknown {
    const obj: any = {}
    message.blockInfo !== undefined &&
      (obj.blockInfo = message.blockInfo
        ? BlockInfo.toJSON(message.blockInfo)
        : undefined)
    message.blockDetails !== undefined &&
      (obj.blockDetails = message.blockDetails
        ? BlockDetails.toJSON(message.blockDetails)
        : undefined)
    if (message.txs) {
      obj.txs = message.txs.map(e => (e ? Tx.toJSON(e) : undefined))
    } else {
      obj.txs = []
    }
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<Block>, I>>(object: I): Block {
    const message = createBaseBlock()
    message.blockInfo =
      object.blockInfo !== undefined && object.blockInfo !== null
        ? BlockInfo.fromPartial(object.blockInfo)
        : undefined
    message.blockDetails =
      object.blockDetails !== undefined && object.blockDetails !== null
        ? BlockDetails.fromPartial(object.blockDetails)
        : undefined
    message.txs = object.txs?.map(e => Tx.fromPartial(e)) || []
    return message
  },
}

function createBaseScriptUtxos(): ScriptUtxos {
  return { outputScript: new Uint8Array(), utxos: [] }
}

export const ScriptUtxos = {
  encode(
    message: ScriptUtxos,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.outputScript.length !== 0) {
      writer.uint32(10).bytes(message.outputScript)
    }
    for (const v of message.utxos) {
      Utxo.encode(v!, writer.uint32(18).fork()).ldelim()
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ScriptUtxos {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseScriptUtxos()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.outputScript = reader.bytes()
          break
        case 2:
          message.utxos.push(Utxo.decode(reader, reader.uint32()))
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): ScriptUtxos {
    return {
      outputScript: isSet(object.outputScript)
        ? bytesFromBase64(object.outputScript)
        : new Uint8Array(),
      utxos: Array.isArray(object?.utxos)
        ? object.utxos.map((e: any) => Utxo.fromJSON(e))
        : [],
    }
  },

  toJSON(message: ScriptUtxos): unknown {
    const obj: any = {}
    message.outputScript !== undefined &&
      (obj.outputScript = base64FromBytes(
        message.outputScript !== undefined
          ? message.outputScript
          : new Uint8Array(),
      ))
    if (message.utxos) {
      obj.utxos = message.utxos.map(e => (e ? Utxo.toJSON(e) : undefined))
    } else {
      obj.utxos = []
    }
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<ScriptUtxos>, I>>(
    object: I,
  ): ScriptUtxos {
    const message = createBaseScriptUtxos()
    message.outputScript = object.outputScript ?? new Uint8Array()
    message.utxos = object.utxos?.map(e => Utxo.fromPartial(e)) || []
    return message
  },
}

function createBaseTxHistoryPage(): TxHistoryPage {
  return { txs: [], numPages: 0 }
}

export const TxHistoryPage = {
  encode(
    message: TxHistoryPage,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    for (const v of message.txs) {
      Tx.encode(v!, writer.uint32(10).fork()).ldelim()
    }
    if (message.numPages !== 0) {
      writer.uint32(16).uint32(message.numPages)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TxHistoryPage {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseTxHistoryPage()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.txs.push(Tx.decode(reader, reader.uint32()))
          break
        case 2:
          message.numPages = reader.uint32()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): TxHistoryPage {
    return {
      txs: Array.isArray(object?.txs)
        ? object.txs.map((e: any) => Tx.fromJSON(e))
        : [],
      numPages: isSet(object.numPages) ? Number(object.numPages) : 0,
    }
  },

  toJSON(message: TxHistoryPage): unknown {
    const obj: any = {}
    if (message.txs) {
      obj.txs = message.txs.map(e => (e ? Tx.toJSON(e) : undefined))
    } else {
      obj.txs = []
    }
    message.numPages !== undefined &&
      (obj.numPages = Math.round(message.numPages))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<TxHistoryPage>, I>>(
    object: I,
  ): TxHistoryPage {
    const message = createBaseTxHistoryPage()
    message.txs = object.txs?.map(e => Tx.fromPartial(e)) || []
    message.numPages = object.numPages ?? 0
    return message
  },
}

function createBaseUtxos(): Utxos {
  return { scriptUtxos: [] }
}

export const Utxos = {
  encode(message: Utxos, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    for (const v of message.scriptUtxos) {
      ScriptUtxos.encode(v!, writer.uint32(10).fork()).ldelim()
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Utxos {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseUtxos()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.scriptUtxos.push(ScriptUtxos.decode(reader, reader.uint32()))
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): Utxos {
    return {
      scriptUtxos: Array.isArray(object?.scriptUtxos)
        ? object.scriptUtxos.map((e: any) => ScriptUtxos.fromJSON(e))
        : [],
    }
  },

  toJSON(message: Utxos): unknown {
    const obj: any = {}
    if (message.scriptUtxos) {
      obj.scriptUtxos = message.scriptUtxos.map(e =>
        e ? ScriptUtxos.toJSON(e) : undefined,
      )
    } else {
      obj.scriptUtxos = []
    }
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<Utxos>, I>>(object: I): Utxos {
    const message = createBaseUtxos()
    message.scriptUtxos =
      object.scriptUtxos?.map(e => ScriptUtxos.fromPartial(e)) || []
    return message
  },
}

function createBaseBlocks(): Blocks {
  return { blocks: [] }
}

export const Blocks = {
  encode(
    message: Blocks,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    for (const v of message.blocks) {
      BlockInfo.encode(v!, writer.uint32(10).fork()).ldelim()
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Blocks {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseBlocks()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.blocks.push(BlockInfo.decode(reader, reader.uint32()))
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): Blocks {
    return {
      blocks: Array.isArray(object?.blocks)
        ? object.blocks.map((e: any) => BlockInfo.fromJSON(e))
        : [],
    }
  },

  toJSON(message: Blocks): unknown {
    const obj: any = {}
    if (message.blocks) {
      obj.blocks = message.blocks.map(e =>
        e ? BlockInfo.toJSON(e) : undefined,
      )
    } else {
      obj.blocks = []
    }
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<Blocks>, I>>(object: I): Blocks {
    const message = createBaseBlocks()
    message.blocks = object.blocks?.map(e => BlockInfo.fromPartial(e)) || []
    return message
  },
}

function createBaseSlpTxData(): SlpTxData {
  return { slpMeta: undefined, genesisInfo: undefined }
}

export const SlpTxData = {
  encode(
    message: SlpTxData,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.slpMeta !== undefined) {
      SlpMeta.encode(message.slpMeta, writer.uint32(10).fork()).ldelim()
    }
    if (message.genesisInfo !== undefined) {
      SlpGenesisInfo.encode(
        message.genesisInfo,
        writer.uint32(18).fork(),
      ).ldelim()
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): SlpTxData {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseSlpTxData()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.slpMeta = SlpMeta.decode(reader, reader.uint32())
          break
        case 2:
          message.genesisInfo = SlpGenesisInfo.decode(reader, reader.uint32())
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): SlpTxData {
    return {
      slpMeta: isSet(object.slpMeta)
        ? SlpMeta.fromJSON(object.slpMeta)
        : undefined,
      genesisInfo: isSet(object.genesisInfo)
        ? SlpGenesisInfo.fromJSON(object.genesisInfo)
        : undefined,
    }
  },

  toJSON(message: SlpTxData): unknown {
    const obj: any = {}
    message.slpMeta !== undefined &&
      (obj.slpMeta = message.slpMeta
        ? SlpMeta.toJSON(message.slpMeta)
        : undefined)
    message.genesisInfo !== undefined &&
      (obj.genesisInfo = message.genesisInfo
        ? SlpGenesisInfo.toJSON(message.genesisInfo)
        : undefined)
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<SlpTxData>, I>>(
    object: I,
  ): SlpTxData {
    const message = createBaseSlpTxData()
    message.slpMeta =
      object.slpMeta !== undefined && object.slpMeta !== null
        ? SlpMeta.fromPartial(object.slpMeta)
        : undefined
    message.genesisInfo =
      object.genesisInfo !== undefined && object.genesisInfo !== null
        ? SlpGenesisInfo.fromPartial(object.genesisInfo)
        : undefined
    return message
  },
}

function createBaseSlpMeta(): SlpMeta {
  return {
    tokenType: 0,
    txType: 0,
    tokenId: new Uint8Array(),
    groupTokenId: new Uint8Array(),
  }
}

export const SlpMeta = {
  encode(
    message: SlpMeta,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.tokenType !== 0) {
      writer.uint32(8).int32(message.tokenType)
    }
    if (message.txType !== 0) {
      writer.uint32(16).int32(message.txType)
    }
    if (message.tokenId.length !== 0) {
      writer.uint32(26).bytes(message.tokenId)
    }
    if (message.groupTokenId.length !== 0) {
      writer.uint32(34).bytes(message.groupTokenId)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): SlpMeta {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseSlpMeta()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.tokenType = reader.int32() as any
          break
        case 2:
          message.txType = reader.int32() as any
          break
        case 3:
          message.tokenId = reader.bytes()
          break
        case 4:
          message.groupTokenId = reader.bytes()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): SlpMeta {
    return {
      tokenType: isSet(object.tokenType)
        ? slpTokenTypeFromJSON(object.tokenType)
        : 0,
      txType: isSet(object.txType) ? slpTxTypeFromJSON(object.txType) : 0,
      tokenId: isSet(object.tokenId)
        ? bytesFromBase64(object.tokenId)
        : new Uint8Array(),
      groupTokenId: isSet(object.groupTokenId)
        ? bytesFromBase64(object.groupTokenId)
        : new Uint8Array(),
    }
  },

  toJSON(message: SlpMeta): unknown {
    const obj: any = {}
    message.tokenType !== undefined &&
      (obj.tokenType = slpTokenTypeToJSON(message.tokenType))
    message.txType !== undefined &&
      (obj.txType = slpTxTypeToJSON(message.txType))
    message.tokenId !== undefined &&
      (obj.tokenId = base64FromBytes(
        message.tokenId !== undefined ? message.tokenId : new Uint8Array(),
      ))
    message.groupTokenId !== undefined &&
      (obj.groupTokenId = base64FromBytes(
        message.groupTokenId !== undefined
          ? message.groupTokenId
          : new Uint8Array(),
      ))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<SlpMeta>, I>>(object: I): SlpMeta {
    const message = createBaseSlpMeta()
    message.tokenType = object.tokenType ?? 0
    message.txType = object.txType ?? 0
    message.tokenId = object.tokenId ?? new Uint8Array()
    message.groupTokenId = object.groupTokenId ?? new Uint8Array()
    return message
  },
}

function createBaseTokenStats(): TokenStats {
  return { totalMinted: "", totalBurned: "" }
}

export const TokenStats = {
  encode(
    message: TokenStats,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.totalMinted !== "") {
      writer.uint32(10).string(message.totalMinted)
    }
    if (message.totalBurned !== "") {
      writer.uint32(18).string(message.totalBurned)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TokenStats {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseTokenStats()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.totalMinted = reader.string()
          break
        case 2:
          message.totalBurned = reader.string()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): TokenStats {
    return {
      totalMinted: isSet(object.totalMinted) ? String(object.totalMinted) : "",
      totalBurned: isSet(object.totalBurned) ? String(object.totalBurned) : "",
    }
  },

  toJSON(message: TokenStats): unknown {
    const obj: any = {}
    message.totalMinted !== undefined && (obj.totalMinted = message.totalMinted)
    message.totalBurned !== undefined && (obj.totalBurned = message.totalBurned)
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<TokenStats>, I>>(
    object: I,
  ): TokenStats {
    const message = createBaseTokenStats()
    message.totalMinted = object.totalMinted ?? ""
    message.totalBurned = object.totalBurned ?? ""
    return message
  },
}

function createBaseTxInput(): TxInput {
  return {
    prevOut: undefined,
    inputScript: new Uint8Array(),
    outputScript: new Uint8Array(),
    value: Long.ZERO,
    sequenceNo: 0,
    slpBurn: undefined,
    slpToken: undefined,
  }
}

export const TxInput = {
  encode(
    message: TxInput,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.prevOut !== undefined) {
      OutPoint.encode(message.prevOut, writer.uint32(10).fork()).ldelim()
    }
    if (message.inputScript.length !== 0) {
      writer.uint32(18).bytes(message.inputScript)
    }
    if (message.outputScript.length !== 0) {
      writer.uint32(26).bytes(message.outputScript)
    }
    if (!message.value.isZero()) {
      writer.uint32(32).int64(message.value)
    }
    if (message.sequenceNo !== 0) {
      writer.uint32(40).uint32(message.sequenceNo)
    }
    if (message.slpBurn !== undefined) {
      SlpBurn.encode(message.slpBurn, writer.uint32(50).fork()).ldelim()
    }
    if (message.slpToken !== undefined) {
      SlpToken.encode(message.slpToken, writer.uint32(58).fork()).ldelim()
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TxInput {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseTxInput()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.prevOut = OutPoint.decode(reader, reader.uint32())
          break
        case 2:
          message.inputScript = reader.bytes()
          break
        case 3:
          message.outputScript = reader.bytes()
          break
        case 4:
          message.value = reader.int64() as Long
          break
        case 5:
          message.sequenceNo = reader.uint32()
          break
        case 6:
          message.slpBurn = SlpBurn.decode(reader, reader.uint32())
          break
        case 7:
          message.slpToken = SlpToken.decode(reader, reader.uint32())
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): TxInput {
    return {
      prevOut: isSet(object.prevOut)
        ? OutPoint.fromJSON(object.prevOut)
        : undefined,
      inputScript: isSet(object.inputScript)
        ? bytesFromBase64(object.inputScript)
        : new Uint8Array(),
      outputScript: isSet(object.outputScript)
        ? bytesFromBase64(object.outputScript)
        : new Uint8Array(),
      value: isSet(object.value) ? Long.fromValue(object.value) : Long.ZERO,
      sequenceNo: isSet(object.sequenceNo) ? Number(object.sequenceNo) : 0,
      slpBurn: isSet(object.slpBurn)
        ? SlpBurn.fromJSON(object.slpBurn)
        : undefined,
      slpToken: isSet(object.slpToken)
        ? SlpToken.fromJSON(object.slpToken)
        : undefined,
    }
  },

  toJSON(message: TxInput): unknown {
    const obj: any = {}
    message.prevOut !== undefined &&
      (obj.prevOut = message.prevOut
        ? OutPoint.toJSON(message.prevOut)
        : undefined)
    message.inputScript !== undefined &&
      (obj.inputScript = base64FromBytes(
        message.inputScript !== undefined
          ? message.inputScript
          : new Uint8Array(),
      ))
    message.outputScript !== undefined &&
      (obj.outputScript = base64FromBytes(
        message.outputScript !== undefined
          ? message.outputScript
          : new Uint8Array(),
      ))
    message.value !== undefined &&
      (obj.value = (message.value || Long.ZERO).toString())
    message.sequenceNo !== undefined &&
      (obj.sequenceNo = Math.round(message.sequenceNo))
    message.slpBurn !== undefined &&
      (obj.slpBurn = message.slpBurn
        ? SlpBurn.toJSON(message.slpBurn)
        : undefined)
    message.slpToken !== undefined &&
      (obj.slpToken = message.slpToken
        ? SlpToken.toJSON(message.slpToken)
        : undefined)
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<TxInput>, I>>(object: I): TxInput {
    const message = createBaseTxInput()
    message.prevOut =
      object.prevOut !== undefined && object.prevOut !== null
        ? OutPoint.fromPartial(object.prevOut)
        : undefined
    message.inputScript = object.inputScript ?? new Uint8Array()
    message.outputScript = object.outputScript ?? new Uint8Array()
    message.value =
      object.value !== undefined && object.value !== null
        ? Long.fromValue(object.value)
        : Long.ZERO
    message.sequenceNo = object.sequenceNo ?? 0
    message.slpBurn =
      object.slpBurn !== undefined && object.slpBurn !== null
        ? SlpBurn.fromPartial(object.slpBurn)
        : undefined
    message.slpToken =
      object.slpToken !== undefined && object.slpToken !== null
        ? SlpToken.fromPartial(object.slpToken)
        : undefined
    return message
  },
}

function createBaseTxOutput(): TxOutput {
  return {
    value: Long.ZERO,
    outputScript: new Uint8Array(),
    slpToken: undefined,
    spentBy: undefined,
  }
}

export const TxOutput = {
  encode(
    message: TxOutput,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (!message.value.isZero()) {
      writer.uint32(8).int64(message.value)
    }
    if (message.outputScript.length !== 0) {
      writer.uint32(18).bytes(message.outputScript)
    }
    if (message.slpToken !== undefined) {
      SlpToken.encode(message.slpToken, writer.uint32(26).fork()).ldelim()
    }
    if (message.spentBy !== undefined) {
      OutPoint.encode(message.spentBy, writer.uint32(34).fork()).ldelim()
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TxOutput {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseTxOutput()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.value = reader.int64() as Long
          break
        case 2:
          message.outputScript = reader.bytes()
          break
        case 3:
          message.slpToken = SlpToken.decode(reader, reader.uint32())
          break
        case 4:
          message.spentBy = OutPoint.decode(reader, reader.uint32())
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): TxOutput {
    return {
      value: isSet(object.value) ? Long.fromValue(object.value) : Long.ZERO,
      outputScript: isSet(object.outputScript)
        ? bytesFromBase64(object.outputScript)
        : new Uint8Array(),
      slpToken: isSet(object.slpToken)
        ? SlpToken.fromJSON(object.slpToken)
        : undefined,
      spentBy: isSet(object.spentBy)
        ? OutPoint.fromJSON(object.spentBy)
        : undefined,
    }
  },

  toJSON(message: TxOutput): unknown {
    const obj: any = {}
    message.value !== undefined &&
      (obj.value = (message.value || Long.ZERO).toString())
    message.outputScript !== undefined &&
      (obj.outputScript = base64FromBytes(
        message.outputScript !== undefined
          ? message.outputScript
          : new Uint8Array(),
      ))
    message.slpToken !== undefined &&
      (obj.slpToken = message.slpToken
        ? SlpToken.toJSON(message.slpToken)
        : undefined)
    message.spentBy !== undefined &&
      (obj.spentBy = message.spentBy
        ? OutPoint.toJSON(message.spentBy)
        : undefined)
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<TxOutput>, I>>(object: I): TxOutput {
    const message = createBaseTxOutput()
    message.value =
      object.value !== undefined && object.value !== null
        ? Long.fromValue(object.value)
        : Long.ZERO
    message.outputScript = object.outputScript ?? new Uint8Array()
    message.slpToken =
      object.slpToken !== undefined && object.slpToken !== null
        ? SlpToken.fromPartial(object.slpToken)
        : undefined
    message.spentBy =
      object.spentBy !== undefined && object.spentBy !== null
        ? OutPoint.fromPartial(object.spentBy)
        : undefined
    return message
  },
}

function createBaseBlockMetadata(): BlockMetadata {
  return { height: 0, hash: new Uint8Array(), timestamp: Long.ZERO }
}

export const BlockMetadata = {
  encode(
    message: BlockMetadata,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.height !== 0) {
      writer.uint32(8).int32(message.height)
    }
    if (message.hash.length !== 0) {
      writer.uint32(18).bytes(message.hash)
    }
    if (!message.timestamp.isZero()) {
      writer.uint32(24).int64(message.timestamp)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): BlockMetadata {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseBlockMetadata()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.height = reader.int32()
          break
        case 2:
          message.hash = reader.bytes()
          break
        case 3:
          message.timestamp = reader.int64() as Long
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): BlockMetadata {
    return {
      height: isSet(object.height) ? Number(object.height) : 0,
      hash: isSet(object.hash)
        ? bytesFromBase64(object.hash)
        : new Uint8Array(),
      timestamp: isSet(object.timestamp)
        ? Long.fromValue(object.timestamp)
        : Long.ZERO,
    }
  },

  toJSON(message: BlockMetadata): unknown {
    const obj: any = {}
    message.height !== undefined && (obj.height = Math.round(message.height))
    message.hash !== undefined &&
      (obj.hash = base64FromBytes(
        message.hash !== undefined ? message.hash : new Uint8Array(),
      ))
    message.timestamp !== undefined &&
      (obj.timestamp = (message.timestamp || Long.ZERO).toString())
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<BlockMetadata>, I>>(
    object: I,
  ): BlockMetadata {
    const message = createBaseBlockMetadata()
    message.height = object.height ?? 0
    message.hash = object.hash ?? new Uint8Array()
    message.timestamp =
      object.timestamp !== undefined && object.timestamp !== null
        ? Long.fromValue(object.timestamp)
        : Long.ZERO
    return message
  },
}

function createBaseOutPoint(): OutPoint {
  return { txid: new Uint8Array(), outIdx: 0 }
}

export const OutPoint = {
  encode(
    message: OutPoint,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.txid.length !== 0) {
      writer.uint32(10).bytes(message.txid)
    }
    if (message.outIdx !== 0) {
      writer.uint32(16).uint32(message.outIdx)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): OutPoint {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseOutPoint()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.txid = reader.bytes()
          break
        case 2:
          message.outIdx = reader.uint32()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): OutPoint {
    return {
      txid: isSet(object.txid)
        ? bytesFromBase64(object.txid)
        : new Uint8Array(),
      outIdx: isSet(object.outIdx) ? Number(object.outIdx) : 0,
    }
  },

  toJSON(message: OutPoint): unknown {
    const obj: any = {}
    message.txid !== undefined &&
      (obj.txid = base64FromBytes(
        message.txid !== undefined ? message.txid : new Uint8Array(),
      ))
    message.outIdx !== undefined && (obj.outIdx = Math.round(message.outIdx))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<OutPoint>, I>>(object: I): OutPoint {
    const message = createBaseOutPoint()
    message.txid = object.txid ?? new Uint8Array()
    message.outIdx = object.outIdx ?? 0
    return message
  },
}

function createBaseSlpToken(): SlpToken {
  return { amount: Long.UZERO, isMintBaton: false }
}

export const SlpToken = {
  encode(
    message: SlpToken,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (!message.amount.isZero()) {
      writer.uint32(8).uint64(message.amount)
    }
    if (message.isMintBaton === true) {
      writer.uint32(16).bool(message.isMintBaton)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): SlpToken {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseSlpToken()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.amount = reader.uint64() as Long
          break
        case 2:
          message.isMintBaton = reader.bool()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): SlpToken {
    return {
      amount: isSet(object.amount) ? Long.fromValue(object.amount) : Long.UZERO,
      isMintBaton: isSet(object.isMintBaton)
        ? Boolean(object.isMintBaton)
        : false,
    }
  },

  toJSON(message: SlpToken): unknown {
    const obj: any = {}
    message.amount !== undefined &&
      (obj.amount = (message.amount || Long.UZERO).toString())
    message.isMintBaton !== undefined && (obj.isMintBaton = message.isMintBaton)
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<SlpToken>, I>>(object: I): SlpToken {
    const message = createBaseSlpToken()
    message.amount =
      object.amount !== undefined && object.amount !== null
        ? Long.fromValue(object.amount)
        : Long.UZERO
    message.isMintBaton = object.isMintBaton ?? false
    return message
  },
}

function createBaseSlpBurn(): SlpBurn {
  return { token: undefined, tokenId: new Uint8Array() }
}

export const SlpBurn = {
  encode(
    message: SlpBurn,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.token !== undefined) {
      SlpToken.encode(message.token, writer.uint32(10).fork()).ldelim()
    }
    if (message.tokenId.length !== 0) {
      writer.uint32(18).bytes(message.tokenId)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): SlpBurn {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseSlpBurn()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.token = SlpToken.decode(reader, reader.uint32())
          break
        case 2:
          message.tokenId = reader.bytes()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): SlpBurn {
    return {
      token: isSet(object.token) ? SlpToken.fromJSON(object.token) : undefined,
      tokenId: isSet(object.tokenId)
        ? bytesFromBase64(object.tokenId)
        : new Uint8Array(),
    }
  },

  toJSON(message: SlpBurn): unknown {
    const obj: any = {}
    message.token !== undefined &&
      (obj.token = message.token ? SlpToken.toJSON(message.token) : undefined)
    message.tokenId !== undefined &&
      (obj.tokenId = base64FromBytes(
        message.tokenId !== undefined ? message.tokenId : new Uint8Array(),
      ))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<SlpBurn>, I>>(object: I): SlpBurn {
    const message = createBaseSlpBurn()
    message.token =
      object.token !== undefined && object.token !== null
        ? SlpToken.fromPartial(object.token)
        : undefined
    message.tokenId = object.tokenId ?? new Uint8Array()
    return message
  },
}

function createBaseSlpGenesisInfo(): SlpGenesisInfo {
  return {
    tokenTicker: new Uint8Array(),
    tokenName: new Uint8Array(),
    tokenDocumentUrl: new Uint8Array(),
    tokenDocumentHash: new Uint8Array(),
    decimals: 0,
  }
}

export const SlpGenesisInfo = {
  encode(
    message: SlpGenesisInfo,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.tokenTicker.length !== 0) {
      writer.uint32(10).bytes(message.tokenTicker)
    }
    if (message.tokenName.length !== 0) {
      writer.uint32(18).bytes(message.tokenName)
    }
    if (message.tokenDocumentUrl.length !== 0) {
      writer.uint32(26).bytes(message.tokenDocumentUrl)
    }
    if (message.tokenDocumentHash.length !== 0) {
      writer.uint32(34).bytes(message.tokenDocumentHash)
    }
    if (message.decimals !== 0) {
      writer.uint32(40).uint32(message.decimals)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): SlpGenesisInfo {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseSlpGenesisInfo()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.tokenTicker = reader.bytes()
          break
        case 2:
          message.tokenName = reader.bytes()
          break
        case 3:
          message.tokenDocumentUrl = reader.bytes()
          break
        case 4:
          message.tokenDocumentHash = reader.bytes()
          break
        case 5:
          message.decimals = reader.uint32()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): SlpGenesisInfo {
    return {
      tokenTicker: isSet(object.tokenTicker)
        ? bytesFromBase64(object.tokenTicker)
        : new Uint8Array(),
      tokenName: isSet(object.tokenName)
        ? bytesFromBase64(object.tokenName)
        : new Uint8Array(),
      tokenDocumentUrl: isSet(object.tokenDocumentUrl)
        ? bytesFromBase64(object.tokenDocumentUrl)
        : new Uint8Array(),
      tokenDocumentHash: isSet(object.tokenDocumentHash)
        ? bytesFromBase64(object.tokenDocumentHash)
        : new Uint8Array(),
      decimals: isSet(object.decimals) ? Number(object.decimals) : 0,
    }
  },

  toJSON(message: SlpGenesisInfo): unknown {
    const obj: any = {}
    message.tokenTicker !== undefined &&
      (obj.tokenTicker = base64FromBytes(
        message.tokenTicker !== undefined
          ? message.tokenTicker
          : new Uint8Array(),
      ))
    message.tokenName !== undefined &&
      (obj.tokenName = base64FromBytes(
        message.tokenName !== undefined ? message.tokenName : new Uint8Array(),
      ))
    message.tokenDocumentUrl !== undefined &&
      (obj.tokenDocumentUrl = base64FromBytes(
        message.tokenDocumentUrl !== undefined
          ? message.tokenDocumentUrl
          : new Uint8Array(),
      ))
    message.tokenDocumentHash !== undefined &&
      (obj.tokenDocumentHash = base64FromBytes(
        message.tokenDocumentHash !== undefined
          ? message.tokenDocumentHash
          : new Uint8Array(),
      ))
    message.decimals !== undefined &&
      (obj.decimals = Math.round(message.decimals))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<SlpGenesisInfo>, I>>(
    object: I,
  ): SlpGenesisInfo {
    const message = createBaseSlpGenesisInfo()
    message.tokenTicker = object.tokenTicker ?? new Uint8Array()
    message.tokenName = object.tokenName ?? new Uint8Array()
    message.tokenDocumentUrl = object.tokenDocumentUrl ?? new Uint8Array()
    message.tokenDocumentHash = object.tokenDocumentHash ?? new Uint8Array()
    message.decimals = object.decimals ?? 0
    return message
  },
}

function createBaseUtxoState(): UtxoState {
  return { height: 0, isConfirmed: false, state: 0 }
}

export const UtxoState = {
  encode(
    message: UtxoState,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.height !== 0) {
      writer.uint32(8).int32(message.height)
    }
    if (message.isConfirmed === true) {
      writer.uint32(16).bool(message.isConfirmed)
    }
    if (message.state !== 0) {
      writer.uint32(24).int32(message.state)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): UtxoState {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseUtxoState()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.height = reader.int32()
          break
        case 2:
          message.isConfirmed = reader.bool()
          break
        case 3:
          message.state = reader.int32() as any
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): UtxoState {
    return {
      height: isSet(object.height) ? Number(object.height) : 0,
      isConfirmed: isSet(object.isConfirmed)
        ? Boolean(object.isConfirmed)
        : false,
      state: isSet(object.state) ? utxoStateVariantFromJSON(object.state) : 0,
    }
  },

  toJSON(message: UtxoState): unknown {
    const obj: any = {}
    message.height !== undefined && (obj.height = Math.round(message.height))
    message.isConfirmed !== undefined && (obj.isConfirmed = message.isConfirmed)
    message.state !== undefined &&
      (obj.state = utxoStateVariantToJSON(message.state))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<UtxoState>, I>>(
    object: I,
  ): UtxoState {
    const message = createBaseUtxoState()
    message.height = object.height ?? 0
    message.isConfirmed = object.isConfirmed ?? false
    message.state = object.state ?? 0
    return message
  },
}

function createBaseSubscription(): Subscription {
  return { scriptType: "", payload: new Uint8Array(), isSubscribe: false }
}

export const Subscription = {
  encode(
    message: Subscription,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.scriptType !== "") {
      writer.uint32(10).string(message.scriptType)
    }
    if (message.payload.length !== 0) {
      writer.uint32(18).bytes(message.payload)
    }
    if (message.isSubscribe === true) {
      writer.uint32(24).bool(message.isSubscribe)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Subscription {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseSubscription()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.scriptType = reader.string()
          break
        case 2:
          message.payload = reader.bytes()
          break
        case 3:
          message.isSubscribe = reader.bool()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): Subscription {
    return {
      scriptType: isSet(object.scriptType) ? String(object.scriptType) : "",
      payload: isSet(object.payload)
        ? bytesFromBase64(object.payload)
        : new Uint8Array(),
      isSubscribe: isSet(object.isSubscribe)
        ? Boolean(object.isSubscribe)
        : false,
    }
  },

  toJSON(message: Subscription): unknown {
    const obj: any = {}
    message.scriptType !== undefined && (obj.scriptType = message.scriptType)
    message.payload !== undefined &&
      (obj.payload = base64FromBytes(
        message.payload !== undefined ? message.payload : new Uint8Array(),
      ))
    message.isSubscribe !== undefined && (obj.isSubscribe = message.isSubscribe)
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<Subscription>, I>>(
    object: I,
  ): Subscription {
    const message = createBaseSubscription()
    message.scriptType = object.scriptType ?? ""
    message.payload = object.payload ?? new Uint8Array()
    message.isSubscribe = object.isSubscribe ?? false
    return message
  },
}

function createBaseSubscribeMsg(): SubscribeMsg {
  return {
    error: undefined,
    AddedToMempool: undefined,
    RemovedFromMempool: undefined,
    Confirmed: undefined,
    Reorg: undefined,
    BlockConnected: undefined,
    BlockDisconnected: undefined,
  }
}

export const SubscribeMsg = {
  encode(
    message: SubscribeMsg,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.error !== undefined) {
      Error.encode(message.error, writer.uint32(10).fork()).ldelim()
    }
    if (message.AddedToMempool !== undefined) {
      MsgAddedToMempool.encode(
        message.AddedToMempool,
        writer.uint32(18).fork(),
      ).ldelim()
    }
    if (message.RemovedFromMempool !== undefined) {
      MsgRemovedFromMempool.encode(
        message.RemovedFromMempool,
        writer.uint32(26).fork(),
      ).ldelim()
    }
    if (message.Confirmed !== undefined) {
      MsgConfirmed.encode(message.Confirmed, writer.uint32(34).fork()).ldelim()
    }
    if (message.Reorg !== undefined) {
      MsgReorg.encode(message.Reorg, writer.uint32(42).fork()).ldelim()
    }
    if (message.BlockConnected !== undefined) {
      MsgBlockConnected.encode(
        message.BlockConnected,
        writer.uint32(50).fork(),
      ).ldelim()
    }
    if (message.BlockDisconnected !== undefined) {
      MsgBlockDisconnected.encode(
        message.BlockDisconnected,
        writer.uint32(58).fork(),
      ).ldelim()
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): SubscribeMsg {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseSubscribeMsg()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.error = Error.decode(reader, reader.uint32())
          break
        case 2:
          message.AddedToMempool = MsgAddedToMempool.decode(
            reader,
            reader.uint32(),
          )
          break
        case 3:
          message.RemovedFromMempool = MsgRemovedFromMempool.decode(
            reader,
            reader.uint32(),
          )
          break
        case 4:
          message.Confirmed = MsgConfirmed.decode(reader, reader.uint32())
          break
        case 5:
          message.Reorg = MsgReorg.decode(reader, reader.uint32())
          break
        case 6:
          message.BlockConnected = MsgBlockConnected.decode(
            reader,
            reader.uint32(),
          )
          break
        case 7:
          message.BlockDisconnected = MsgBlockDisconnected.decode(
            reader,
            reader.uint32(),
          )
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): SubscribeMsg {
    return {
      error: isSet(object.error) ? Error.fromJSON(object.error) : undefined,
      AddedToMempool: isSet(object.AddedToMempool)
        ? MsgAddedToMempool.fromJSON(object.AddedToMempool)
        : undefined,
      RemovedFromMempool: isSet(object.RemovedFromMempool)
        ? MsgRemovedFromMempool.fromJSON(object.RemovedFromMempool)
        : undefined,
      Confirmed: isSet(object.Confirmed)
        ? MsgConfirmed.fromJSON(object.Confirmed)
        : undefined,
      Reorg: isSet(object.Reorg) ? MsgReorg.fromJSON(object.Reorg) : undefined,
      BlockConnected: isSet(object.BlockConnected)
        ? MsgBlockConnected.fromJSON(object.BlockConnected)
        : undefined,
      BlockDisconnected: isSet(object.BlockDisconnected)
        ? MsgBlockDisconnected.fromJSON(object.BlockDisconnected)
        : undefined,
    }
  },

  toJSON(message: SubscribeMsg): unknown {
    const obj: any = {}
    message.error !== undefined &&
      (obj.error = message.error ? Error.toJSON(message.error) : undefined)
    message.AddedToMempool !== undefined &&
      (obj.AddedToMempool = message.AddedToMempool
        ? MsgAddedToMempool.toJSON(message.AddedToMempool)
        : undefined)
    message.RemovedFromMempool !== undefined &&
      (obj.RemovedFromMempool = message.RemovedFromMempool
        ? MsgRemovedFromMempool.toJSON(message.RemovedFromMempool)
        : undefined)
    message.Confirmed !== undefined &&
      (obj.Confirmed = message.Confirmed
        ? MsgConfirmed.toJSON(message.Confirmed)
        : undefined)
    message.Reorg !== undefined &&
      (obj.Reorg = message.Reorg ? MsgReorg.toJSON(message.Reorg) : undefined)
    message.BlockConnected !== undefined &&
      (obj.BlockConnected = message.BlockConnected
        ? MsgBlockConnected.toJSON(message.BlockConnected)
        : undefined)
    message.BlockDisconnected !== undefined &&
      (obj.BlockDisconnected = message.BlockDisconnected
        ? MsgBlockDisconnected.toJSON(message.BlockDisconnected)
        : undefined)
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<SubscribeMsg>, I>>(
    object: I,
  ): SubscribeMsg {
    const message = createBaseSubscribeMsg()
    message.error =
      object.error !== undefined && object.error !== null
        ? Error.fromPartial(object.error)
        : undefined
    message.AddedToMempool =
      object.AddedToMempool !== undefined && object.AddedToMempool !== null
        ? MsgAddedToMempool.fromPartial(object.AddedToMempool)
        : undefined
    message.RemovedFromMempool =
      object.RemovedFromMempool !== undefined &&
      object.RemovedFromMempool !== null
        ? MsgRemovedFromMempool.fromPartial(object.RemovedFromMempool)
        : undefined
    message.Confirmed =
      object.Confirmed !== undefined && object.Confirmed !== null
        ? MsgConfirmed.fromPartial(object.Confirmed)
        : undefined
    message.Reorg =
      object.Reorg !== undefined && object.Reorg !== null
        ? MsgReorg.fromPartial(object.Reorg)
        : undefined
    message.BlockConnected =
      object.BlockConnected !== undefined && object.BlockConnected !== null
        ? MsgBlockConnected.fromPartial(object.BlockConnected)
        : undefined
    message.BlockDisconnected =
      object.BlockDisconnected !== undefined &&
      object.BlockDisconnected !== null
        ? MsgBlockDisconnected.fromPartial(object.BlockDisconnected)
        : undefined
    return message
  },
}

function createBaseMsgAddedToMempool(): MsgAddedToMempool {
  return { txid: new Uint8Array() }
}

export const MsgAddedToMempool = {
  encode(
    message: MsgAddedToMempool,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.txid.length !== 0) {
      writer.uint32(10).bytes(message.txid)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): MsgAddedToMempool {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseMsgAddedToMempool()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.txid = reader.bytes()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): MsgAddedToMempool {
    return {
      txid: isSet(object.txid)
        ? bytesFromBase64(object.txid)
        : new Uint8Array(),
    }
  },

  toJSON(message: MsgAddedToMempool): unknown {
    const obj: any = {}
    message.txid !== undefined &&
      (obj.txid = base64FromBytes(
        message.txid !== undefined ? message.txid : new Uint8Array(),
      ))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<MsgAddedToMempool>, I>>(
    object: I,
  ): MsgAddedToMempool {
    const message = createBaseMsgAddedToMempool()
    message.txid = object.txid ?? new Uint8Array()
    return message
  },
}

function createBaseMsgRemovedFromMempool(): MsgRemovedFromMempool {
  return { txid: new Uint8Array() }
}

export const MsgRemovedFromMempool = {
  encode(
    message: MsgRemovedFromMempool,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.txid.length !== 0) {
      writer.uint32(10).bytes(message.txid)
    }
    return writer
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number,
  ): MsgRemovedFromMempool {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseMsgRemovedFromMempool()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.txid = reader.bytes()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): MsgRemovedFromMempool {
    return {
      txid: isSet(object.txid)
        ? bytesFromBase64(object.txid)
        : new Uint8Array(),
    }
  },

  toJSON(message: MsgRemovedFromMempool): unknown {
    const obj: any = {}
    message.txid !== undefined &&
      (obj.txid = base64FromBytes(
        message.txid !== undefined ? message.txid : new Uint8Array(),
      ))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<MsgRemovedFromMempool>, I>>(
    object: I,
  ): MsgRemovedFromMempool {
    const message = createBaseMsgRemovedFromMempool()
    message.txid = object.txid ?? new Uint8Array()
    return message
  },
}

function createBaseMsgConfirmed(): MsgConfirmed {
  return { txid: new Uint8Array() }
}

export const MsgConfirmed = {
  encode(
    message: MsgConfirmed,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.txid.length !== 0) {
      writer.uint32(10).bytes(message.txid)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): MsgConfirmed {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseMsgConfirmed()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.txid = reader.bytes()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): MsgConfirmed {
    return {
      txid: isSet(object.txid)
        ? bytesFromBase64(object.txid)
        : new Uint8Array(),
    }
  },

  toJSON(message: MsgConfirmed): unknown {
    const obj: any = {}
    message.txid !== undefined &&
      (obj.txid = base64FromBytes(
        message.txid !== undefined ? message.txid : new Uint8Array(),
      ))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<MsgConfirmed>, I>>(
    object: I,
  ): MsgConfirmed {
    const message = createBaseMsgConfirmed()
    message.txid = object.txid ?? new Uint8Array()
    return message
  },
}

function createBaseMsgReorg(): MsgReorg {
  return { txid: new Uint8Array() }
}

export const MsgReorg = {
  encode(
    message: MsgReorg,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.txid.length !== 0) {
      writer.uint32(10).bytes(message.txid)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): MsgReorg {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseMsgReorg()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.txid = reader.bytes()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): MsgReorg {
    return {
      txid: isSet(object.txid)
        ? bytesFromBase64(object.txid)
        : new Uint8Array(),
    }
  },

  toJSON(message: MsgReorg): unknown {
    const obj: any = {}
    message.txid !== undefined &&
      (obj.txid = base64FromBytes(
        message.txid !== undefined ? message.txid : new Uint8Array(),
      ))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<MsgReorg>, I>>(object: I): MsgReorg {
    const message = createBaseMsgReorg()
    message.txid = object.txid ?? new Uint8Array()
    return message
  },
}

function createBaseMsgBlockConnected(): MsgBlockConnected {
  return { blockHash: new Uint8Array() }
}

export const MsgBlockConnected = {
  encode(
    message: MsgBlockConnected,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.blockHash.length !== 0) {
      writer.uint32(10).bytes(message.blockHash)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): MsgBlockConnected {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseMsgBlockConnected()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.blockHash = reader.bytes()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): MsgBlockConnected {
    return {
      blockHash: isSet(object.blockHash)
        ? bytesFromBase64(object.blockHash)
        : new Uint8Array(),
    }
  },

  toJSON(message: MsgBlockConnected): unknown {
    const obj: any = {}
    message.blockHash !== undefined &&
      (obj.blockHash = base64FromBytes(
        message.blockHash !== undefined ? message.blockHash : new Uint8Array(),
      ))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<MsgBlockConnected>, I>>(
    object: I,
  ): MsgBlockConnected {
    const message = createBaseMsgBlockConnected()
    message.blockHash = object.blockHash ?? new Uint8Array()
    return message
  },
}

function createBaseMsgBlockDisconnected(): MsgBlockDisconnected {
  return { blockHash: new Uint8Array() }
}

export const MsgBlockDisconnected = {
  encode(
    message: MsgBlockDisconnected,
    writer: _m0.Writer = _m0.Writer.create(),
  ): _m0.Writer {
    if (message.blockHash.length !== 0) {
      writer.uint32(10).bytes(message.blockHash)
    }
    return writer
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number,
  ): MsgBlockDisconnected {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseMsgBlockDisconnected()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.blockHash = reader.bytes()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): MsgBlockDisconnected {
    return {
      blockHash: isSet(object.blockHash)
        ? bytesFromBase64(object.blockHash)
        : new Uint8Array(),
    }
  },

  toJSON(message: MsgBlockDisconnected): unknown {
    const obj: any = {}
    message.blockHash !== undefined &&
      (obj.blockHash = base64FromBytes(
        message.blockHash !== undefined ? message.blockHash : new Uint8Array(),
      ))
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<MsgBlockDisconnected>, I>>(
    object: I,
  ): MsgBlockDisconnected {
    const message = createBaseMsgBlockDisconnected()
    message.blockHash = object.blockHash ?? new Uint8Array()
    return message
  },
}

function createBaseError(): Error {
  return { errorCode: "", msg: "", isUserError: false }
}

export const Error = {
  encode(message: Error, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.errorCode !== "") {
      writer.uint32(10).string(message.errorCode)
    }
    if (message.msg !== "") {
      writer.uint32(18).string(message.msg)
    }
    if (message.isUserError === true) {
      writer.uint32(24).bool(message.isUserError)
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Error {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseError()
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          message.errorCode = reader.string()
          break
        case 2:
          message.msg = reader.string()
          break
        case 3:
          message.isUserError = reader.bool()
          break
        default:
          reader.skipType(tag & 7)
          break
      }
    }
    return message
  },

  fromJSON(object: any): Error {
    return {
      errorCode: isSet(object.errorCode) ? String(object.errorCode) : "",
      msg: isSet(object.msg) ? String(object.msg) : "",
      isUserError: isSet(object.isUserError)
        ? Boolean(object.isUserError)
        : false,
    }
  },

  toJSON(message: Error): unknown {
    const obj: any = {}
    message.errorCode !== undefined && (obj.errorCode = message.errorCode)
    message.msg !== undefined && (obj.msg = message.msg)
    message.isUserError !== undefined && (obj.isUserError = message.isUserError)
    return obj
  },

  fromPartial<I extends Exact<DeepPartial<Error>, I>>(object: I): Error {
    const message = createBaseError()
    message.errorCode = object.errorCode ?? ""
    message.msg = object.msg ?? ""
    message.isUserError = object.isUserError ?? false
    return message
  },
}

declare var self: any | undefined
declare var window: any | undefined
declare var global: any | undefined
var globalThis: any = (() => {
  if (typeof globalThis !== "undefined") return globalThis
  if (typeof self !== "undefined") return self
  if (typeof window !== "undefined") return window
  if (typeof global !== "undefined") return global
  throw "Unable to locate global object"
})()

const atob: (b64: string) => string =
  globalThis.atob ||
  (b64 => globalThis.Buffer.from(b64, "base64").toString("binary"))
function bytesFromBase64(b64: string): Uint8Array {
  const bin = atob(b64)
  const arr = new Uint8Array(bin.length)
  for (let i = 0; i < bin.length; ++i) {
    arr[i] = bin.charCodeAt(i)
  }
  return arr
}

const btoa: (bin: string) => string =
  globalThis.btoa ||
  (bin => globalThis.Buffer.from(bin, "binary").toString("base64"))
function base64FromBytes(arr: Uint8Array): string {
  const bin: string[] = []
  arr.forEach(byte => {
    bin.push(String.fromCharCode(byte))
  })
  return btoa(bin.join(""))
}

type Builtin =
  | Date
  | Function
  | Uint8Array
  | string
  | number
  | boolean
  | undefined

export type DeepPartial<T> = T extends Builtin
  ? T
  : T extends Long
  ? string | number | Long
  : T extends Array<infer U>
  ? Array<DeepPartial<U>>
  : T extends ReadonlyArray<infer U>
  ? ReadonlyArray<DeepPartial<U>>
  : T extends {}
  ? { [K in keyof T]?: DeepPartial<T[K]> }
  : Partial<T>

type KeysOfUnion<T> = T extends T ? keyof T : never
export type Exact<P, I extends P> = P extends Builtin
  ? P
  : P & { [K in keyof P]: Exact<P[K], I[K]> } & Record<
        Exclude<keyof I, KeysOfUnion<P>>,
        never
      >

if (_m0.util.Long !== Long) {
  _m0.util.Long = Long as any
  _m0.configure()
}

function isSet(value: any): boolean {
  return value !== null && value !== undefined
}
