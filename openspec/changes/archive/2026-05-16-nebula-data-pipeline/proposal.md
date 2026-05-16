# Proposition : Pipeline d'Évaluation de la Divergence (Data Pipeline)

## Contexte
Pour générer un dataset de qualité (Active Learning) destiné à la forge Nebula, il faut isoler mathématiquement les moments où un agent de l'essaim Pulsar hallucine à cause d'un manque d'information. Les LLMs ne pouvant évaluer leur propre incertitude de manière fiable, ce pipeline met en place une validation déterministe hors-bande (Out-of-Band) basée sur la variance des réponses générées à différentes températures.

## Objectifs
Implémenter une chaîne de 4 FaaS WebAssembly autonomes, situés dans le répertoire `faas/` à la racine du projet :

1. **`nebula-telemetry-gateway`** : Point d'entrée allégé écoutant le bus d'événements de Tachyon pour capturer les triplets de réponses (1x T=0.1, 2x T=0.8).
2. **`nebula-eval-ast`** : Moteur d'évaluation syntaxique basé sur Tree-sitter pour le code source, détectant les variations logiques (hachage structurel).
3. **`nebula-eval-semantic`** : Moteur d'évaluation sémantique utilisant la similarité cosinus via des petits modèles d'embedding (Candle) pour le texte libre ou comme filet de sécurité.
4. **`nebula-divergence-aggregator`** : Collecteur final qui assemble les cas d'échec avérés et les pousse dans la file d'attente du modèle professeur (Tier 3).

## Architecture de communication
Les FaaS communiqueront exclusivement via le composant `tachyon:messaging/event-bus`, garantissant un découplage total et la possibilité de mettre à l'échelle les FaaS vectoriels indépendamment des FaaS syntaxiques.