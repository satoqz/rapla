package main

import (
	"github.com/satoqz/rapla-proxy/internal/server"
)

func main() {
	if err := server.Serve(); err != nil {
		panic(err)
	}
}
