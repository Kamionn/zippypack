# Architecture de ZippyPack

**Créé par : Kamion (Matthéo Le Fur)**  
**Date : 14/07/2025**  
**Version : 1.0.0**

## Vue d'ensemble

ZippyPack est architecturé autour de plusieurs modules spécialisés qui collaborent pour offrir une compression avancée avec déduplication par blocs.

## Structure des modules

### 🏗️ Core Modules

#### `src/main.rs`
- **Rôle** : Interface CLI principale
- **Responsabilités** : Parsing des arguments, dispatch des commandes
- **Dépendances** : clap, env_logger

#### `src/lib.rs`
- **Rôle** : Exposition publique des modules
- **Responsabilités** : Organisation des exports, tests

#### `src/compress.rs`
- **Rôle** : Compression traditionnelle (.zpp)
- **Responsabilités** : Compression par dossiers, détection de types
- **Algorithmes** : zstd, solid compression

#### `src/decompress.rs`
- **Rôle** : Décompression des archives .zpp
- **Responsabilités** : Restauration des fichiers, validation d'intégrité
- **Sécurité** : Sanitization des chemins

#### `src/image.rs` 🚀
- **Rôle** : Système d'images avec déduplication
- **Responsabilités** : Création/extraction d'images .zpak
- **Innovation** : Déduplication par blocs de 64KB

#### `src/profile.rs`
- **Rôle** : Profils de compression par type
- **Responsabilités** : Optimisation contextuelle
- **Types supportés** : Text, Binary, GameEngine, etc.

#### `src/error.rs`
- **Rôle** : Gestion d'erreurs typée
- **Responsabilités** : Définition des erreurs spécifiques

## Flux de données

### Compression traditionnelle
```
Dossier → Scan files → Type detection → Compression → .zpp
```

### Système d'images
```
Dossier → Scan files → Block splitting → Deduplication → Index → .zpak
```

### Décompression
```
.zpp/.zpak → Read index → Decompress blocks → Restore files
```

## Formats de fichiers

### Format .zpp (Compression traditionnelle)
1. **Header** : Taille dictionnaire (8 bytes)
2. **Dictionnaire** : Données du dictionnaire zstd
3. **Données compressées** : Flux zstd solid

### Format .zpak (Système d'images)
1. **Header** : Version, stats, métadonnées (48 bytes)
2. **Index des blocs** : Hash + position + taille de chaque bloc
3. **Données compressées** : Blocs zstd dédupliqués
4. **Métadonnées fichiers** : Arborescence + références aux blocs

## Algorithmes clés

### Déduplication par blocs
- **Taille de bloc** : 64KB (65536 bytes)
- **Hash** : DefaultHasher (simple mais efficace)
- **Stockage** : HashMap<BlockHash, DataBlock>

### Compression zstd
- **Niveaux** : 1-22 (défaut: 22)
- **Mode solid** : Disponible pour .zpp
- **Dictionnaires** : Génération automatique

## Performances

### Complexité temporelle
- **Compression** : O(n) avec n = taille totale
- **Déduplication** : O(n/64KB) pour l'indexation
- **Décompression** : O(n) linéaire

### Complexité spatiale
- **Mémoire** : O(nombre de blocs uniques)
- **Stockage** : O(données uniques après déduplication)

## Extensibilité

### Ajout de nouveaux formats
1. Créer un nouveau module dans `src/`
2. Définir les structures Options
3. Implémenter les fonctions create/extract
4. Ajouter les commandes CLI dans `main.rs`

### Nouveaux algorithmes
1. Modifier `compress.rs` pour l'intégration
2. Ajouter les profils dans `profile.rs`
3. Mettre à jour les tests

## Sécurité

### Sanitization
- **Chemins** : Validation des caractères Windows/Unix
- **Taille** : Limites sur les blocs et fichiers
- **Intégrité** : Checksums sur les données critiques

### Vulnérabilités atténuées
- **Path traversal** : Nettoyage des chemins relatifs
- **Zip bombs** : Limites de décompression
- **Memory exhaustion** : Streaming des gros fichiers

## Tests

### Structure des tests
- `src/tests/compression_tests.rs` : Tests unitaires
- `examples/` : Exemples d'utilisation
- `tools/` : Utilitaires de test

### Couverture
- ✅ Compression/décompression basic
- ✅ Système d'images
- ✅ Round-trip integrity
- ✅ Error handling

---

Cette architecture permet une évolution modulaire tout en maintenant des performances optimales et une sécurité robuste.