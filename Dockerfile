FROM rust:latest

WORKDIR /usr/src/holysee

COPY . .

RUN make install

WORKDIR /usr/src/holysee/holysee

CMD ["holysee"]
