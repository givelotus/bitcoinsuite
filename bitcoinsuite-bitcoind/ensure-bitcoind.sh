#!/bin/bash
set -ue

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
dloc="${SCRIPT_DIR}/../downloads"
abc_url_osx="https://download.bitcoinabc.org/latest/osx/"
abc_url_linux="https://download.bitcoinabc.org/latest/linux/"
bch_url_linux="https://github.com/bitcoin-cash-node/bitcoin-cash-node/releases/download/v24.0.0/bitcoin-cash-node-24.0.0-x86_64-linux-gnu.tar.gz"
lotusd_version="4.3.3"
lotusd_url_osx="https://storage.googleapis.com/lotus-project/lotus-$lotusd_version-osx64.tar.gz"
lotusd_url_linux="https://storage.googleapis.com/lotus-project/lotus-$lotusd_version-x86_64-linux-gnu.tar.gz"
mkdir -p "$dloc"/bitcoin-cash-node
mkdir -p "$dloc"/bitcoin-abc
mkdir -p "$dloc"/lotusd
touch "$dloc/abcversion"
touch "$dloc/bchversion"

osxbch() {
    # Download OSX BCH
    OSXBCH=$(
        curl -s "https://github.com/bitcoin-cash-node/bitcoin-cash-node/releases" |\
            sed -n -E '
                /<a href="\/bitcoin-cash/{
                    /osx64/{
                        s/[[:space:]]*<a href="([^"]*)".*/\1/
                        p
                        q
                    }
                }'
    )
    filename=${OSXBCH##*/}
    : ${oldfilename:=}
    read oldfilename < "$dloc/bchversion" || true
    if [ "$oldfilename" = "$filename" ]
    then
        echo "BCHN already latest version"
        return
    fi
    curl --create-dirs --output-dir "$dloc/bitcoin-cash-node/" -LO "https://github.com$OSXBCH"
    tar xvf $dloc/bitcoin-cash-node/*tar.gz -C $dloc/bitcoin-cash-node/
    mv $dloc/bitcoin-cash-node/*/* $dloc/bitcoin-cash-node/
    find $dloc/bitcoin-cash-node/ -empty -type d -delete
    rm $dloc/bitcoin-cash-node/*tar.gz
    echo "$filename" > "$dloc/bchversion"
}

osxabc() {
    # Download OSX ABC
    OSXABC=$(curl -s $abc_url_osx |\
        sed -E -n '/osx64/{
        s/<[^>]*>([^<]*).*/\1/p
        }'
    )
    : ${oldfilename:=}
    read oldfilename < "$dloc/abcversion" || true
    if [ "$oldfilename" = "$OSXABC" ]
    then
        echo "Bitcoin ABC already latest version"
        return
    fi
    curl --create-dirs --output-dir "$dloc/bitcoin-abc/" -LO "$abc_url_osx""$OSXABC"
    tar xvf $dloc/bitcoin-abc/*tar.gz -C $dloc/bitcoin-abc
    mv $dloc/bitcoin-abc/*/* $dloc/bitcoin-abc/
    find $dloc/bitcoin-abc/ -empty -type d -delete
    rm $dloc/bitcoin-abc/*tar.gz
    echo "$OSXABC" > "$dloc/abcversion"
}

linuxbch() {
    # Download Linux BCH
    : ${oldfilename:=}
    read oldfilename < "$dloc/bchversion" || true
    if [ "$oldfilename" = "$linuxABC" ]
    then
        echo "bitcoin ABC already latest version"
        return
    fi

    curl --create-dirs --output-dir "$dloc/bitcoin-cash-node/" -LO "$bch_url_linux"
    tar xvf $dloc/bitcoin-cash-node/*tar.gz -C $dloc/bitcoin-cash-node
    mv $dloc/bitcoin-cash-node/*/* $dloc/bitcoin-cash-node/
    find $dloc/bitcoin-cash-node/ -empty -type d -delete
    rm $dloc/bitcoin-cash-node/*tar.gz
    echo "$bch_url_linux" > "$dloc/abcversion"
}

linuxabc() {
    # Download OSX ABC
    linuxABC=$(curl -s $abc_url_linux |\
        sed -E -n '/x86_64-linux-gnu.tar/{
        s/<[^>]*>([^<]*).*/\1/p
        }'
    )
    : ${oldfilename:=}
    read oldfilename < "$dloc/abcversion" || true
    if [ "$oldfilename" = "$linuxABC" ]
    then
        echo "bitcoin ABC already latest version"
        return
    fi

    curl --create-dirs --output-dir "$dloc/bitcoin-abc/" -LO "$abc_url_linux""$linuxABC"
    tar xvf $dloc/bitcoin-abc/*tar.gz -C $dloc/bitcoin-abc
    mv $dloc/bitcoin-abc/*/* $dloc/bitcoin-abc/
    find $dloc/bitcoin-abc/ -empty -type d -delete
    rm $dloc/bitcoin-abc/*tar.gz
    echo "$linuxABC" > "$dloc/abcversion"
}

osxlotusd() {
    : ${oldversion:=}
    read oldversion < "$dloc/lotusversion" || true
    if [ "$oldversion" = "$lotusd_version" ]
    then
        echo "Lotusd already expected version"
        return
    fi

    curl --create-dirs --output-dir "$dloc/lotusd/" -LO "$lotusd_url_osx" 
    tar xvf $dloc/lotusd/*tar.gz -C $dloc/lotusd
    mv $dloc/lotusd/*/* $dloc/lotusd/
    find $dloc/lotusd/ -empty -type d -delete
    rm $dloc/lotusd/*tar.gz
    echo "$lotusd_version" > "$dloc/lotusversion"
}

linuxlotusd() {
    curl --create-dirs --output-dir "$dloc/lotusd/" -LO "$lotusd_url_linux" 
    tar xvf $dloc/lotusd/*tar.gz -C $dloc/lotusd
    mv $dloc/lotusd/*/* $dloc/lotusd/
    find $dloc/lotusd/ -empty -type d -delete
    rm $dloc/lotusd/*tar.gz
    echo "$lotusd_version" > "$dloc/lotusversion"
}

if [ "$(uname -s)" = "Darwin" ];then
    osxabc
    osxbch
    osxlotusd
else
    linuxabc
    linuxbch
    linuxlotusd
fi
