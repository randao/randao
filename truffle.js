module.exports = {
  "build": {
    // Copy ./app/index.html (right hand side) to ./build/index.html.
    "index.html": "index.html",
    "app.js": [
      // Paths relative to "app" directory that should be
      // concatenated and processed during build.
      "javascripts/app.js"
    ],
    "app.css": [
      // Paths relative to "app" directory that should be
      // concatenated and processed during build.
      "stylesheets/app.css"
    ]
  },
  "deploy": [
    // Names of contracts that should be deployed to the network.
    "Randao",
    "Counter",
    "Sha3"
  ],
  "rpc": {
    // Default RPC configuration.
    "host": "127.0.0.1",
    "port": 4500
  }, 
  "networks": {
    "morden": {
      network_id: 2,
      port: 8545
    },
    "development": {
      network_id: "default"
    }
  }
}
