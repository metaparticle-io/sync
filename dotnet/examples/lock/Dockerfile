FROM debian:9
RUN apt-get update && apt-get -qq -y install libunwind8 libicu57 libssl1.0 liblttng-ust0 libcurl3 libuuid1 libkrb5-3 zlib1g
COPY bin/release/netcoreapp2.0/debian.8-x64/publish/* /exe/
CMD /exe/lock
