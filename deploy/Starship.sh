#!/bin/sh
# Name: Starship
# Author: Primitive AI
# Icon: data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAQAAAAEACAAAAAB5Gfe6AAADfElEQVR42u2d0ZKsIAxEaWr+/5ezL3v3VumAgM4Aycnz6tLHNkrASUoEQRAEQRBEzNDwkeZDh/bXfk+MvIgfFSRX6gc0yZ38TlUalq+lJA+PTiP/QIte+JFBqvvMWtr83SNV30m1QQLoG656TriD/N4Rq/1su8jvG7Q8yu8ZuNpOs5v89rFnt/r/Bm3jDrCN5bcKyJ71/xu6jQHYX38LAV3o31p+i47sXf+vBOsF4Eb/FYHsXv8Fgexff51ATsEjBzBA1QI5gv4agRxCf4UAOSCGAcoWyE8vGe62VpD3Xf14Zkk3RzFASVSOZYCzwNdzBrDVvaOG9wD3BjhJzFEyQElYjmaAo8gcyABvpb1W2G+1zKtwiDvgIPO174UcmbTImA0CoAzAnD8D/uQZDgDAJ94D9tnnjQPeA4iQA89ZEAcAAAAAAAAAAAAAADAZSp8rROEAAACAgsiq32HigBgLYDgAAAAAAAAAAAAAAAAAjlaHn9msyy3ALTDrS4VF6gM4IAX/cUSSIAAAAAAAAAAAAAAAAAAAAGaDM2duhgMAAAAAAAAAAAAAAAAAAAAAAAAAWK4eYKO9OdJ3vicY3V/ALQAAAAAAAAAAAAAAAAAAAAAAAKAe8L021LZIz3NuAQAAAAAAAAAAAAAAAAAAAAAAAPWAtMz3AHePN+EAAAAAAACIC4BdYgAAAAAAAAAAAAAAT9UDdHN//uzjcQAAAAAAAAAAAAAAAAAAAAAAAAAA9/cH3P39/9nH4wAAAAAAAAAAAAAAAAAAAAAAAMD/CaZ5F3zsSokDAACA2KG7bWs3a7d7+lOqwk91k9VGx9ZzgOs3Aas7QPbpRtfr5b1X1Ha7lSRoke6AAwClFOzJ//YxaIEMcMwBPWlQLhJGTnEsYC1JULEyQOlV2MIY4AxAsQxQmgxZFAO8ASCfBKzgbypCKYYFSgZ46wCHBIr6C0nfnD0QKnrIAZWnpQUwQMkBrghUb+ic3BOoJ7ScvBO4SOi6fHeUhxdgjSyNycG04Poi5uSZQIOJ9ZXll7nzP42vDmvn2XHb1Wtu5Khdp/8XA9djZ9pSfpMq265c1jNi9RWTtFftq2G46q6naZvKX9NINVJS1A51z8ZB6mONgOfWfJtHpwV6HaWZGwa0RLunifsllBZpeTVrt4hm9BlbaauM0qxua+tsEyIIgiAIgiBCxg8Oo5nmiFvr5QAAAABJRU5ErkJggg==
# DontUseFBInk
# PrimitiveUSBLease

app=/mnt/us/starship/bin/starship-kindle
[ -x "$app" ] || exit 1
if [ -n "${KL_NATIVE_APP_RUN:-}" ]; then
    cp "$app" "$KL_NATIVE_APP_RUN/run" && chmod 755 "$KL_NATIVE_APP_RUN/run" || exit 1
    cd / || exit 1
    exec "$KL_NATIVE_APP_RUN/run"
fi

# Older launchers and the stock library start this entry without a shared RAM work directory.
case "$0" in
    /var/tmp/primitive-starship.*/launch) work=${0%/launch} ;;
    *)
        work=$(mktemp -d /var/tmp/primitive-starship.XXXXXX) || exit 1
        if ! cp "$0" "$work/launch"; then
            rmdir "$work"
            exit 1
        fi
        cd / || exit 1
        exec /bin/sh "$work/launch"
        ;;
esac
trap 'rm -rf "$work"' EXIT
cp "$app" "$work/run" && chmod 755 "$work/run" || exit 1
"$work/run"
