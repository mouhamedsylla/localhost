# 🚀 Localhost: Because Every Server Deserves Some Style! 

## 🌟 Welcome to the Party!

Hey there, awesome developer! You've just stumbled upon **Localhost**, the HTTP server that makes serving web content as fun as playing with LEGO blocks (but with more ports and fewer foot injuries). Whether you're building a small personal project or a large-scale application, Localhost is here to make your life easier, faster, and more stylish.

---

## 🎭 What's This Magic All About?

**Localhost** is a high-performance, customizable HTTP server built with **Rust** that combines power, flexibility, and a touch of humor. It’s designed to handle everything from serving static files to executing CGI scripts, managing file uploads, and handling sessions. Think of it as your very own digital butler who juggles requests with ease and style.

---

## 🎮 Let's Get This Show on the Road!

### Installation

Getting started with Localhost is as easy as pie. Here’s how you can clone, build, and run it:

```bash
# Clone our fantastic repository
git clone https://github.com/yourusername/Localhost.git

# Step into the magic zone
cd Localhost

# Build it like you mean it!
cargo build --release
```

### Running the Server

Once built, you can run Localhost in several ways depending on your needs:

```bash
# The simple way
./Localhost

# The "I'm a pro" way (with a custom config file)
./Localhost -c config.json

# The "show me everything" way (with warnings enabled)
./Localhost --warn  # Because warnings are like spoilers for server problems!
```

---

## 🎨 The Art of Configuration

Localhost is highly configurable via a simple JSON configuration file. Here’s a sample configuration to get you started:

```json
{
  "servers": [
    {
      "server_address": "127.0.0.2",
      "server_name": "server1",  // Give it a cool name!
      "ports": ["8080"],         // Ports are like doors to your server party
      "client_body_size_limit": "10M",  // Because size matters
      "session": {
        "enabled": true,
        "name": "session_id",    // Keep it classy
        "options": {
          "max_age": 86400,      // One day of fun!
          "domain": "server1.home",
          "path": "/",
          "secure": false,       // Living on the edge (just kidding, secure it in production!)
          "http_only": true,
          "same_site": "Lax"
        }
      }
    }
  ]
}
```

---

## 🎪 Routes: Where the Magic Happens

Routes in Localhost are defined in the configuration file and allow you to specify how different paths are handled. Here’s an example:

```json
{
  "path": "/home",               // The VIP entrance
  "methods": ["GET", "POST", "DELETE"],  // The party tricks we know
  "root": "static",             // Where we keep the good stuff
  "default_page": "index.html", // The welcome mat
  "directory_listing": true,    // Let's show off a bit
  "session_required": true,     // No ticket, no entry!
  "session_redirect": "/"       // The walk of shame
}
```

---

## 🎯 API Endpoints: What Can You Do?

Localhost exposes a powerful API to handle various tasks. Here’s a breakdown of the endpoints and what they allow you to do:

### **Static File Handling**
- **`GET /static/*`**  
  Serve static files (HTML, CSS, JS, images, etc.) from the specified directory. Perfect for hosting your frontend assets.

---

### **File Upload and Management**
- **`GET /api/files/list`**  
  List all uploaded files with their metadata (ID, name, size, etc.).
  
- **`POST /api/files/upload`**  
  Upload one or multiple files to the server. Supports multipart form data.

- **`DELETE /api/files/:id`**  
  Delete a specific file by its ID. Useful for cleaning up unused resources.

---

### **CGI Script Execution**
- **`GET /cgi-bin/*`**  
  Execute CGI scripts located in the specified directory. Ideal for dynamic content generation.

---

### **Session Management**
- **`POST /api/session/create`**  
  Create a new session for a user. Returns a session ID that can be used for subsequent requests.

- **`DELETE /api/session/delete`**  
  Destroy an existing session. Useful for logging users out or cleaning up expired sessions.

---

## 🎭 Warning Mode: For the Drama Lovers

Run Localhost with the `--warn` flag, and it will become your most honest critic. It’ll catch things like:

- **Duplicate server names**: *"Hey, those server names are twins!"*
- **Overlapping routes**: *"Your routes are playing musical chairs!"*
- **Missing error pages**: *"404 page missing - now that's a real 404!"*

---

## 🌈 The Directory Fashion Show

Here’s the trendy directory layout for Localhost:

```
Localhost/
├── config.json          # The master plan
├── static/             # The public gallery
├── error/              # Where mistakes look good
└── example/            # Show and tell
```

---

## 🎪 Performance: Speed is Our Middle Name

Localhost is like a caffeinated cheetah wearing rocket boots - it's seriously fast! Thanks to:

- **Keep-alive connections** (because goodbyes are overrated)
- **Non-blocking I/O** (we don't like waiting either)
- **Smart session management** (we remember the cool kids)

---

## 🎉 The Grand Finale

Remember: Every great server starts with a single request. Make it count!

**Made with ❤️, 🦀 Rust, and a sprinkle of server magic.**

---

## 🛠️ Need Help? Found a Bug? Want to Contribute?

We’re like a pineapple on pizza - always ready to create controversy! Open an issue or submit a PR. We’d love to hear from you!

**Licensed under MIT** - because sharing is caring! 🎈

---

P.S. If you're still reading this, you're officially awesome! Now go build something incredible with Localhost! 🚀
