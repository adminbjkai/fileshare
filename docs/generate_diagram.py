#!/usr/bin/env python3
"""Generate architecture diagram for FileShare application."""

from diagrams import Diagram, Cluster, Edge
from diagrams.onprem.client import User
from diagrams.onprem.network import Nginx
from diagrams.programming.language import Rust
from diagrams.generic.storage import Storage
from diagrams.generic.compute import Rack
import os

# Set output directory
os.chdir(os.path.dirname(os.path.abspath(__file__)))

graph_attr = {
    "bgcolor": "white",
    "fontcolor": "#333333",
    "fontsize": "14",
    "pad": "0.5",
    "splines": "ortho",
}

node_attr = {
    "fontcolor": "#333333",
    "fontsize": "11",
}

edge_attr = {
    "color": "#666666",
    "fontcolor": "#333333",
    "fontsize": "10",
}

with Diagram(
    "FileShare Architecture",
    filename="architecture_diagram",
    show=False,
    direction="TB",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
    outformat="png",
):
    user = User("Browser\nClient")

    with Cluster("Internet", graph_attr={"bgcolor": "#e8f4f8", "fontcolor": "#1a5276"}):
        nginx = Nginx("Nginx\nReverse Proxy\nfileshare.bjk.ai:443")

    with Cluster("Server (Port 7012)", graph_attr={"bgcolor": "#fef9e7", "fontcolor": "#7d6608"}):
        with Cluster("Actix-web Application", graph_attr={"bgcolor": "#fff5eb", "fontcolor": "#a04000"}):
            rust_app = Rust("Actix-web\nServer")

        with Cluster("Handlers", graph_attr={"bgcolor": "#eafaf1", "fontcolor": "#1e8449"}):
            upload = Rack("POST /upload\nFile Handler")
            download = Rack("GET /{file}\nFile Streamer")
            qr = Rack("GET /qr\nQR Generator")

        with Cluster("Modules", graph_attr={"bgcolor": "#f5eef8", "fontcolor": "#6c3483"}):
            zip_mod = Rack("ZIP\nCompressor")
            qr_mod = Rack("QR Code\nEncoder")

    with Cluster("Storage", graph_attr={"bgcolor": "#fdedec", "fontcolor": "#922b21"}):
        storage = Storage("./uploads/\nFile Storage")
        counter = Storage(".zipcount\nID Counter")

    # Connections
    user >> Edge(label="HTTPS", color="#2980b9") >> nginx
    nginx >> Edge(label="HTTP:7012", color="#2980b9") >> rust_app

    rust_app >> Edge(color="#27ae60") >> upload
    rust_app >> Edge(color="#27ae60") >> download
    rust_app >> Edge(color="#27ae60") >> qr

    upload >> Edge(color="#8e44ad") >> zip_mod
    qr >> Edge(color="#8e44ad") >> qr_mod

    upload >> Edge(label="write", color="#c0392b") >> storage
    download >> Edge(label="read", color="#c0392b") >> storage
    zip_mod >> Edge(label="read/write", color="#c0392b") >> counter

print("Diagram generated: architecture_diagram.png")
