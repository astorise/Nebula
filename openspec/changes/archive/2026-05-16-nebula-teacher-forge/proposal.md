# Proposition : Forge de Dataset et Arbitrage Tier 3

## Contexte
Une fois les zones d'incertitudes et d'hallucinations des agents locaux (Tier 1/2) identifiées par le pipeline d'évaluation, ces impasses doivent être résolues par un modèle disposant d'un raisonnement supérieur (Teacher Model / Tier 3, ex: DeepSeek). Ce processus doit s'exécuter sous de fortes contraintes matérielles (VRAM limitée) et aboutir à la création automatisée de fichiers de poids neuronaux (LoRA).

## Objectifs
Implémenter la chaîne de distillation et d'entraînement dans `faas/` :

1. **`nebula-curriculum-generator`** : FaaS initiant des examens synthétiques de connaissances (ex: Cobol, Rust) pour tester les petits modèles de manière proactive (sans documentation).
2. **`nebula-teacher-arbitrator`** : Le FaaS lourd qui orchestre le chargement progressif (layer-by-layer) du modèle Tier 3 via l'interface `tachyon:inference` pour corriger les hallucinations.
3. **`nebula-dataset-forge`** : Le FaaS de stockage qui assemble les paires contrastives dans un volume persistant (`.jsonl`) tout en garantissant le ratio de confiance (60% échec résolu / 40% succès direct).
4. **`nebula-training-orchestrator`** : FaaS déclenché à un seuil défini (ex: 500 exemples) pour piloter l'entraînement de la couche LoRA et sa publication sur le registre d'artefacts local.