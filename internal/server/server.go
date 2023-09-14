package server

import (
	"log/slog"
	"net/http"

	"github.com/satoqz/rapla-proxy/pkg/rapla"
)

func handler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	cal := rapla.Extract("woo")
	cal.SerializeTo(w)
}

func Serve() error {
	http.HandleFunc("/", handler)
	server := http.Server{
		Addr: "127.0.0.1:8080",
	}

	slog.Info("HTTP server listening", "addr", server.Addr)
	return server.ListenAndServe()
}
