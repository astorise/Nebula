# Proposition : Fondation de l'Architecture Nebula

## Contexte
Nebula agit comme la "forge d'entraînement" de l'écosystème. L'architecture nécessite une interface utilisateur familière (VSCode) couplée à un moteur d'orchestration local (CLI Node.js) capable de s'interfacer avec le maillage Tachyon et de lire des données contextuelles sans risquer de les altérer.

## Objectifs
- **CLI Node.js (Le Moteur) :**
  - Exposer un serveur **WebDAV en lecture seule**. Il servira de point de montage virtuel pour ingérer la documentation locale (PDF, Markdown, DOCX) pour le LLM Tier 3 (Teacher), bloquant par design toute suppression ou modification par hallucination.
  - Exposer un serveur **WebSocket** pour maintenir une communication bidirectionnelle temps réel avec les fonctions FaaS Wasm déployées sur Tachyon.
- **Extension VSCode (L'Interface) :**
  - Reprendre l'ergonomie de Pulsar pour piloter les workflows d'apprentissage et visualiser la génération de dataset et le fine-tuning LoRA.
- **Sécurité (Workflow d'apprentissage) :**
  - Sécuriser rigoureusement les endpoints (WebDAV et WebSocket) via un **Certificat Client (mTLS)** dans l'immédiat, préparant le terrain pour un flux OAuth2 futur.