#!/bin/bash
dloc="../downloads"
abc_url_osx="https://download.bitcoinabc.org/latest/osx/"
abc_url_linux="https://download.bitcoinabc.org/latest/linux/"
lotusd_version="3.3.3"
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
    read oldfilename < "$dloc/bchversion"

    if [ "$oldfilename" = "$filename" ]
    then
        echo "bitcoin cash node already latest version"
    else
        wget -q --show-progress -P "$dloc/bitcoin-cash-node/" "https://github.com$OSXBCH"
        tar xvf $dloc/bitcoin-cash-node/*tar.gz -C $dloc/bitcoin-cash-node/
        mv $dloc/bitcoin-cash-node/*/* $dloc/bitcoin-cash-node/
        find $dloc/bitcoin-cash-node/ -empty -type d -delete
        rm $dloc/bitcoin-cash-node/*tar.gz

    fi

    echo "$filename" > "$dloc/bchversion"
}

osxabc() {
    # Download OSX ABC
    OSXABC=$(curl -s $abc_url_osx |\
        sed -E -n '/osx64/{
        s/<[^>]*>([^<]*).*/\1/p
        }'
    )

    read oldfilename < "$dloc/abcversion"
    if [ "$oldfilename" = "$OSXABC" ]
    then
        echo "bitcoin ABC already latest version"
    else
        wget -q --show-progress -P "$dloc/bitcoin-abc/" "$abc_url_osx""$OSXABC"
        tar xvf $dloc/bitcoin-abc/*tar.gz -C $dloc/bitcoin-abc
        mv $dloc/bitcoin-abc/*/* $dloc/bitcoin-abc/
        find $dloc/bitcoin-abc/ -empty -type d -delete
        rm $dloc/bitcoin-abc/*tar.gz
    fi

    echo "$OSXABC" > "$dloc/abcversion"
}

linuxbch() {
    # Download Linux BCH
    linuxBCH=$(
        curl -s "https://github.com/bitcoin-cash-node/bitcoin-cash-node/releases" |\
            sed -n -E '
                /<a href="\/bitcoin-cash/{
                    /x86_64-linux-gnu.tar/{
                        s/[[:space:]]*<a href="([^"]*)".*/\1/
                        p
                        q
                    }
                }'
    )
    
    filename=${linuxBCH##*/}

    read oldfilename < "$dloc/bchversion"

    if [ "$oldfilename" = "$filename" ]
    then
        echo "bitcoin cash node already latest version"
    else
        wget -q --show-progress -P "$dloc/bitcoin-cash-node/" "https://github.com$linuxBCH"
        tar xvf $dloc/bitcoin-cash-node/*tar.gz -C $dloc/bitcoin-cash-node
        mv $dloc/bitcoin-cash-node/*/* $dloc/bitcoin-cash-node/
        find $dloc/bitcoin-cash-node/ -empty -type d -delete
        rm $dloc/bitcoin-cash-node/*tar.gz
    fi

    echo "$filename" > "$dloc/bchversion"
}

linuxabc() {
    # Download OSX ABC
    linuxABC=$(curl -s $abc_url_linux |\
        sed -E -n '/x86_64-linux-gnu.tar/{
        s/<[^>]*>([^<]*).*/\1/p
        }'
    )
    
    read oldfilename < "$dloc/abcversion"
    if [ "$oldfilename" = "$linuxABC" ]
    then
        echo "bitcoin ABC already latest version"
    else
        wget -q --show-progress -P "$dloc/bitcoin-abc/" "$abc_url_linux""$linuxABC"
        tar xvf $dloc/bitcoin-abc/*tar.gz -C $dloc/bitcoin-abc
        mv $dloc/bitcoin-abc/*/* $dloc/bitcoin-abc/
        find $dloc/bitcoin-abc/ -empty -type d -delete
        rm $dloc/bitcoin-abc/*tar.gz
    fi

    echo "$linuxABC" > "$dloc/abcversion"
}

linuxlotusd() {
    wget -P "$dloc/lotusd/" "$lotusd_url_linux"
    tar xvf $dloc/lotusd/*tar.gz -C $dloc/lotusd
    mv $dloc/lotusd/*/* $dloc/lotusd/
    find $dloc/lotusd/ -empty -type d -delete
    rm $dloc/lotusd/*tar.gz
}

if [ "$(uname -s)" = "Darwin" ];then
    osxabc
    osxbch
else
    linuxabc
    linuxbch
    linuxlotusd
fi
