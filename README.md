# 🚀 Localhost: Because Every Server Deserves Some Style! [![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/yourusername/localhost) [![License](https://img.shields.io/badge/license-MIT-blue)](https://opensource.org/licenses/MIT) [![Rust](https://img.shields.io/badge/rust-1.72%2B-orange)](https://www.rust-lang.org/)

<div align="center">
  <img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Objects/Desktop%20Computer.png" alt="Desktop Computer" width="150" />
  
  <p><i>A high-performance HTTP server with personality</i></p>
  
  [![Stars](https://img.shields.io/github/stars/mouhamedsylla/localhost?style=social)](https://github.com/yourusername/localhost)
  [![Forks](https://img.shields.io/github/forks/mouhamedsylla/localhost?style=social)](https://github.com/yourusername/localhost/fork)
  [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/yourusername/localhost/pulls)
  [![Made with Love](https://img.shields.io/badge/Made%20with-♥-ff69b4)]()
</div>

## 🌟 Welcome to the Party!

Hey there, awesome developer! You've just stumbled upon **Localhost**, the HTTP server that makes serving web content as fun as playing with LEGO blocks (but with more ports and fewer foot injuries). Whether you're building a small personal project or a large-scale application, Localhost is here to make your life easier, faster, and more stylish.

<div align="center">
  <img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Hand%20gestures/Waving%20Hand.png" alt="Waving Hand" width="80" />
</div>

---

## 🎭 What's This Magic All About?

**Localhost** is a high-performance, customizable HTTP server built with **Rust** that combines power, flexibility, and a touch of humor. It features:

<table>
  <tr>
    <td><img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Travel%20and%20places/Globe%20Showing%20Europe-Africa.png" width="40"/></td>
    <td><b>Virtual Hosting</b>: Run multiple domains from a single server instance</td>
  </tr>
  <tr>
    <td><img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Objects/File%20Folder.png" width="40"/></td>
    <td><b>Static File Serving</b>: Lightning-fast file delivery with directory listing option</td>
  </tr>
  <tr>
    <td><img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Objects/Gear.png" width="40"/></td>
    <td><b>CGI Support</b>: Run dynamic scripts for server-side processing</td>
  </tr>
  <tr>
    <td><img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Objects/Key.png" width="40"/></td>
    <td><b>Session Management</b>: Built-in user session handling</td>
  </tr>
  <tr>
    <td><img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Objects/Inbox%20Tray.png" width="40"/></td>
    <td><b>File Upload API</b>: Easy file management through REST endpoints</td>
  </tr>
  <tr>
    <td><img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Symbols/Warning.png" width="40"/></td>
    <td><b>Custom Error Pages</b>: Make even your 404s look good</td>
  </tr>
</table>

All powered by an event-driven, non-blocking architecture with epoll for maximum efficiency.

---

## 🎮 Let's Get This Show on the Road! [![Installation](https://img.shields.io/badge/Difficulty-Easy-success)](https://github.com/yourusername/localhost#installation)

### Installation

Getting started with Localhost is as easy as pie. Here's how you can clone, build, and run it:

```bash
# Clone our fantastic repository
git clone https://github.com/yourusername/localhost.git

# Step into the magic zone
cd localhost

# Build it like you mean it!
cargo build --release
```

### Running the Server [![Server Status](https://img.shields.io/badge/Status-Running-success)](https://github.com/yourusername/localhost#running-the-server)

Once built, you can run Localhost like this:

```bash
# The simple way
./target/release/localhost

# The "show me everything" way (with warnings enabled)
./target/release/localhost --warn  # Because warnings are like spoilers for server problems!
```

<div align="center">
  <img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Travel%20and%20places/Rocket.png" alt="Rocket" width="80" />
</div>

## 🎨 The Art of Configuration [![Config](https://img.shields.io/badge/Config-JSON-yellow)](https://github.com/yourusername/localhost#configuration)

Localhost is configured via a JSON file (`src/config/config.json` by default). Here's what a typical configuration looks like:

```json
{
  "servers": [
    {
      "server_address": "127.0.0.2",
      "server_name": "server1.home",  // Give it a cool name!
      "ports": ["8080"],              // Ports are like doors to your server party
      "client_body_size_limit": "10M",  // Because size matters
      "session": {
        "enabled": true,
        "name": "session_id",         // Keep it classy
        "options": {
          "max_age": 86400,           // One day of fun!
          "domain": "server1.home",
          "path": "/",
          "secure": false,            // Secure it in production!
          "http_only": true,
          "same_site": "Lax"
        }
      },
      "error_pages": {
        "custom_pages": {
          "404": "error/404.html"     // Style your errors
        }
      }
    }
  ]
}
```
Each server entry defines a virtual host with its own configuration.

<div align="center">
  <img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Hand%20gestures/Writing%20Hand.png" alt="Writing Hand" width="80" />
</div>

## 🧩 Virtual Hosting: Multiple Personalities Welcome! [![Hosts](https://img.shields.io/badge/Virtual%20Hosts-Unlimited-blueviolet)](https://github.com/yourusername/localhost#virtual-hosting)

One of Localhost's superpowers is virtual hosting - running multiple websites on a single server:

```json
{
  "servers": [
    {
      "server_address": "127.0.0.2",
      "server_name": "server1.home",
      "ports": ["8080"]
      // other config...
    },
    {
      "server_address": "127.0.0.3",
      "server_name": "server2.home",
      "ports": ["8080"]
      // different config...
    }
  ]
}
```

Each virtual host:

- Has its own domain name
- Can listen on multiple ports
- Has independent routes and configurations
- Can have custom error pages

The server automatically adds entries to your hosts file for convenient local development.

## ⚙️ How It Works: Behind the Curtain [![Architecture](https://img.shields.io/badge/Architecture-Event--Driven-informational)](https://github.com/yourusername/localhost#how-it-works)

Localhost uses an event-driven architecture powered by epoll:

<div align="center">
  <img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Objects/Magnifying%20Glass%20Tilted%20Left.png" alt="Magnifying Glass" width="80" />
</div>

1. **Event Loop**: Efficiently waits for network events
2. **Connection Handling**: Non-blocking I/O for maximum throughput
3. **Request Parsing**: Fast HTTP parsing with sensible timeouts
4. **Route Matching**: Directs requests to appropriate handlers
5. **Response Generation**: Delivers content with proper headers

All of this happens asynchronously without blocking threads, giving you maximum performance even under heavy load.

## 🗂️ Project Structure: Finding Your Way Around [![Structure](https://img.shields.io/badge/Structure-Organized-success)](https://github.com/yourusername/localhost#project-structure)

```tree
localhost/  
├── src/                # Code source  
│   ├── server/        # Fonctionnalités principales du serveur  
│   │   ├── http/       # Implémentation du protocole HTTP  
│   │   ├── core/       # Gestion du serveur et des fonctionnalités de base  
│   │   │   ├── host.rs       # Implémentation des hôtes virtuels  
│   │   │   ├── handlers.rs   # Gestionnaires de requêtes  
│   │   │   ├── session.rs    # Gestion des sessions  
│   │   │   └── ...           # Autres composants du serveur  
│   ├── config/       # Gestion de la configuration  
├── static/            # Fichiers statiques  
├── uploads/           # Répertoire des fichiers téléchargés  
├── errors/            # Pages d'erreur personnalisées  
└── config.json        # Configuration du serveur  
```

---

## 📝 Technical Details: For the Curious Minds [![Tech Stack](https://img.shields.io/badge/Tech%20Stack-Rust-orange)](https://github.com/yourusername/localhost#technical-details)

<div align="center">
  <table>
    <tr>
      <td align="center"><img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Objects/Wrench.png" width="40"/></td>
      <td><b>Language</b>: Rust</td>
    </tr>
    <tr>
      <td align="center"><img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Objects/Gear.png" width="40"/></td>
      <td><b>Architecture</b>: Event-driven with epoll</td>
    </tr>
    <tr>
      <td align="center"><img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Travel%20and%20places/Globe%20with%20Meridians.png" width="40"/></td>
      <td><b>HTTP Support</b>: HTTP/1.1 with keep-alive</td>
    </tr>
    <tr>
      <td align="center"><img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Objects/Link.png" width="40"/></td>
      <td><b>Connection Model</b>: Non-blocking I/O</td>
    </tr>
    <tr>
      <td align="center"><img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Objects/Key.png" width="40"/></td>
      <td><b>Session Storage</b>: In-memory with configurable options</td>
    </tr>
    <tr>
      <td align="center"><img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Objects/File%20Folder.png" width="40"/></td>
      <td><b>File Handling</b>: Optimized static file serving with directory listings</td>
    </tr>
    <tr>
      <td align="center"><img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Objects/Page%20with%20Curl.png" width="40"/></td>
      <td><b>CGI Support</b>: Execute dynamic scripts in multiple languages</td>
    </tr>
    <tr>
      <td align="center"><img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Objects/Card%20File%20Box.png" width="40"/></td>
      <td><b>Configuration</b>: JSON-based with extensive validation</td>
    </tr>
  </table>
</div>

<div align="center">
  <a href="https://github.com/yourusername/localhost/stargazers">
    <img src="https://img.shields.io/github/stars/yourusername/localhost?style=for-the-badge&color=yellow" alt="Stars" />
  </a>
  <a href="https://github.com/yourusername/localhost/network/members">
    <img src="https://img.shields.io/github/forks/yourusername/localhost?style=for-the-badge&color=orange" alt="Forks" />
  </a>
  <a href="https://github.com/yourusername/localhost/issues">
    <img src="https://img.shields.io/github/issues/yourusername/localhost?style=for-the-badge&color=red" alt="Issues" />
  </a>
</div>

<div align="center">
  <p>Made with ❤️ by passionate developers</p>
  <img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Objects/Party%20Popper.png" alt="Party Popper" width="100" />
</div>