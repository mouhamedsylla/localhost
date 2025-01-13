# Localhost

# 🚀 Serveur Web Modulaire en Rust

Bienvenue dans ce projet de serveur web moderne et performant écrit en Rust ! Ce serveur a été conçu pour être modulaire, efficace et facile à étendre.

## 📑 Table des matières

- [Architecture](#architecture)
- [Fonctionnalités](#fonctionnalités)
- [Composants principaux](#composants-principaux)
- [Flux de fonctionnement](#flux-de-fonctionnement)
- [Configuration](#configuration)
- [Utilisation](#utilisation)
- [Contribuer](#contribuer)

## 🏗 Architecture

Le serveur est construit autour d'une architecture événementielle utilisant epoll, permettant une excellente performance même sous forte charge. Voici le diagramme de flux principal :

```mermaid
flowchart TD
    %% Main Entry Points
    Start([Client Request]) --> Epoll{Epoll Event Loop}
    
    %% Event Types
    Epoll -->|New Connection| NC[Handle New Connection]
    Epoll -->|Data Available| DR[Read Request Data]
    Epoll -->|Timeout Check| TC[Check Connection Timeouts]
    
    %% New Connection Flow
    NC --> FindHost{Find Host by FD}
    FindHost -->|Not Found| Error1[Log Error]
    FindHost -->|Found| Accept[Accept Connection]
    Accept --> SetNB[Set Non-Blocking]
    SetNB --> AddEpoll[Add to Epoll]
    AddEpoll --> Store[Store Connection]
    
    %% Request Processing Flow
    DR --> Parse{Parse Request}
    Parse -->|Invalid| Close1[Close Connection]
    Parse -->|Valid| Route{Route Request}
    
    %% Routing Logic
    Route -->|/api/*| API[File API Handler]
    Route -->|Static File| Static[Static File Handler]
    Route -->|CGI Script| CGI[CGI Handler]
    Route -->|Not Found| NF[Not Found Response]
    
    %% API Handler Flow
    API --> ApiMethod{HTTP Method}
    ApiMethod -->|GET| ListFiles[List Files]
    ApiMethod -->|POST| Upload[Handle Upload]
    ApiMethod -->|DELETE| Delete[Delete File]
    
    %% Static Handler Flow
    Static --> CheckFile{Check File}
    CheckFile -->|Exists| ServeFile[Serve File]
    CheckFile -->|Not Found| NF
    
    %% CGI Handler Flow
    CGI --> ValidScript{Valid Script?}
    ValidScript -->|Yes| Exec[Execute Script]
    ValidScript -->|No| Error2[Error Response]
    Exec --> ParseOut[Parse Output]
    
    %% Response Handling
    ServeFile --> Send[Send Response]
    Upload --> Send
    Delete --> Send
    ListFiles --> Send
    ParseOut --> Send
    Error2 --> Send
    NF --> Send
    
    %% Connection Management
    Send --> KeepAlive{Keep-Alive?}
    KeepAlive -->|Yes| Epoll
    KeepAlive -->|No| Close2[Close Connection]
    
    %% Timeout Management
    TC --> CheckTime{Timeout?}
    CheckTime -->|Yes| Close3[Close Connection]
    CheckTime -->|No| Epoll
```

## ✨ Fonctionnalités

- **Virtual Hosting** : Support de plusieurs domaines sur un même serveur
- **Gestion de fichiers statiques** : Serveur de fichiers optimisé
- **Support CGI** : Exécution de scripts dynamiques
- **API de gestion de fichiers** : Upload, listing et suppression via REST API
- **Keep-Alive** : Connexions persistantes pour de meilleures performances
- **Logging intégré** : Suivi détaillé des opérations
- **Gestion des timeouts** : Protection contre les connexions zombies

## 🧩 Composants principaux

### Server
Le cœur du système qui :
- Gère la boucle epoll principale
- Maintient les connexions actives
- Dispatche les événements aux bons handlers
- Assure la surveillance des timeouts

### Host
Gère la configuration des hôtes virtuels avec :
- Support multi-ports
- Configuration des routes
- Gestion des listeners

### Connection
Gère les connexions individuelles :
- Buffer de lecture optimisé
- Parse des requêtes HTTP
- Gestion du keep-alive

### Handlers
Trois types de handlers spécialisés :

1. **StaticFileHandler**
   - Sert les fichiers statiques
   - Gestion du cache et des types MIME
   - Support des répertoires

2. **CGIHandler**
   - Exécution sécurisée de scripts
   - Gestion des variables d'environnement
   - Parse des sorties CGI

3. **FileAPIHandler**
   - API RESTful pour la gestion de fichiers
   - Upload multipart
   - Listing et suppression de fichiers

## 🔄 Flux de fonctionnement

1. **Réception d'une requête**
   - La boucle epoll détecte une nouvelle connexion
   - Le serveur accepte et configure la socket
   - La connexion est ajoutée au système de surveillance

2. **Traitement de la requête**
   - Lecture des données via un buffer optimisé
   - Parse de la requête HTTP
   - Identification du handler approprié

3. **Routing**
   - Matching de l'URL avec les routes configurées
   - Sélection du handler approprié
   - Transmission de la requête

4. **Génération de la réponse**
   - Traitement par le handler spécialisé
   - Construction de la réponse HTTP
   - Envoi au client

5. **Gestion de la connexion**
   - Vérification du keep-alive
   - Mise à jour des timers
   - Fermeture si nécessaire

## ⚙️ Configuration

Configuration via un fichier YAML :

```yaml
hosts:
  - server_name: example.com
    address: "127.0.0.1"
    ports: ["80", "8080"]
    routes:
      - path: "/static"
        static_files:
          root: "./public"
      - path: "/cgi-bin"
        cgi:
          interpreter: "/usr/bin/python3"
          script_dir: "./scripts"
```

## 🚀 Utilisation

Pour démarrer le serveur :

```bash
cargo run --release -- --config config.yml
```

## 🤝 Contribuer

Les contributions sont les bienvenues ! Voici comment participer :

1. Fork le projet
2. Créez une nouvelle branche (`git checkout -b feature/awesome-feature`)
3. Committez vos changements (`git commit -am 'Add awesome feature'`)
4. Push sur la branche (`git push origin feature/awesome-feature`)
5. Ouvrez une Pull Request

---

📝 **Note** : Ce projet est en développement actif. N'hésitez pas à ouvrir des issues pour des bugs ou des suggestions d'amélioration !




```mermaid
stateDiagram-v2
    [*] --> EventLoop : Démarrage du serveur

    state EventLoop {
        [*] --> WaitForEvents : epoll_wait()
        
        WaitForEvents --> HandleEvent : Événement disponible
        
        state HandleEvent {
            [*] --> CheckEventType
            
            state AcceptConnection {
                [*] --> CheckNewConnection
                CheckNewConnection --> TryAccept : Nouvelle connexion TCP
                CheckNewConnection --> NoMoreConnections : Plus de connexions
                
                TryAccept --> InitializeConnection
                InitializeConnection --> WaitHTTPRequest
                
                NoMoreConnections --> [*]
            }
            
            state HTTPRequestHandling {
                [*] --> ParseHTTPRequest
                
                ParseHTTPRequest --> CheckHTTPMethod
                
                state CheckHTTPMethod {
                    [*] --> HandleGET : Méthode GET
                    [*] --> HandlePOST : Méthode POST
                    [*] --> HandlePUT : Méthode PUT
                    [*] --> HandleDELETE : Méthode DELETE
                    [*] --> HandlePATCH : Méthode PATCH
                }
                
                state HandleGET {
                    [*] --> RouteGET
                    RouteGET --> PrepareResponse
                    PrepareResponse --> SendResponse
                    SendResponse --> [*]
                }
                
                state HandlePOST {
                    [*] --> ParseBody
                    ParseBody --> ValidateData
                    ValidateData --> ProcessCreate
                    ProcessCreate --> PrepareResponse
                    PrepareResponse --> SendResponse
                    SendResponse --> [*]
                }
                
                state HandlePUT {
                    [*] --> ParseBody
                    ParseBody --> ValidateData
                    ValidateData --> ProcessUpdate
                    ProcessUpdate --> PrepareResponse
                    PrepareResponse --> SendResponse
                    SendResponse --> [*]
                }
                
                state HandleDELETE {
                    [*] --> RouteResource
                    RouteResource --> ValidateDelete
                    ValidateDelete --> ProcessDelete
                    ProcessDelete --> PrepareResponse
                    PrepareResponse --> SendResponse
                    SendResponse --> [*]
                }
                
                state HandlePATCH {
                    [*] --> ParseBody
                    ParseBody --> ValidateData
                    ValidateData --> ProcessPartialUpdate
                    ProcessPartialUpdate --> PrepareResponse
                    PrepareResponse --> SendResponse
                    SendResponse --> [*]
                }
                
                SendResponse --> CloseConnection
                CloseConnection --> [*]
            }
            
            CheckEventType --> HTTPRequestHandling : Données à lire
        }
        
        HandleEvent --> WaitForEvents : Retour à l'attente
    }
    
    EventLoop --> [*] : Arrêt du serveur

    note right of HTTPRequestHandling
        Traitement non-bloquant:
        - Parse rapide de la requête
        - Routing sans attente
        - Réponse immédiate
        - Gestion des différents verbes HTTP
    end note
```