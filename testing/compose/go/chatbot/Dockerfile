FROM golang:1.25-alpine AS build
WORKDIR /repo
COPY bindings/heeczer-go /repo/bindings/heeczer-go
COPY testing/compose/go/chatbot /repo/testing/compose/go/chatbot
COPY testing/tests/fixtures /fixtures
WORKDIR /repo/testing/compose/go/chatbot
RUN go mod download
RUN go build -o /chatbot .

FROM alpine:3.21
COPY --from=build /chatbot /usr/local/bin/chatbot
COPY --from=build /fixtures /fixtures
EXPOSE 8000
CMD ["/usr/local/bin/chatbot"]
