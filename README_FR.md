# ZippyPack

**ZippyPack** est un outil de compression avancé en Rust qui utilise l'algorithme Zstandard avec déduplication par blocs et format d'image système.

## 🚀 Fonctionnalités

- **Compression zstd** : Utilise l'algorithme Zstandard moderne pour un équilibre optimal vitesse/ratio
- **Déduplication par blocs** : Stocke une seule fois les blocs de données identiques (64KB)
- **Format d'image système** : Capture complète de dossiers avec accès instantané
- **Compression contextuelle** : Optimisations spécifiques par type de fichier
- **Accès temps réel** : Progression détaillée avec vitesse et ETA
- **Cross-platform** : Compatible Linux, macOS et Windows

## 📊 Performances

Sur un dataset de 505 fichiers de code source :
- **Ratio de compression** : 95.67% (5.1 MB → 222 KB)
- **Comparaison** : 6% d'écart avec WinRAR, 12% mieux que 7-Zip
- **Vitesse** : ~0.2 MB/s avec compression maximale

## 📁 Structure du projet

```
zippypack/
├── src/                    # Code source principal
│   ├── main.rs            # Interface CLI
│   ├── lib.rs             # Bibliothèque publique
│   ├── compress.rs        # Compression traditionnelle
│   ├── decompress.rs      # Décompression
│   ├── image.rs           # Système d'images avec déduplication
│   ├── profile.rs         # Profils de compression
│   └── error.rs           # Gestion d'erreurs
├── examples/              # Exemples d'utilisation
├── tools/                 # Utilitaires de développement
├── docs/                  # Documentation technique
└── README.md             # Ce fichier
```

## 🔧 Installation

```bash
git clone https://github.com/kamionn/zippypack.git
cd zippypack
cargo build --release
```

## 📖 Utilisation

### Compression classique (.zpp)
```bash
# Comprimer un dossier
cargo run --release -- compress --input dossier/ --output archive.zpp --level 22

# Décompresser une archive
cargo run --release -- decompress --input archive.zpp --output dossier_restauré/
```

### Image système (.zpak)
```bash
# Créer une image système avec déduplication
cargo run --release -- create-image --input projet/ --output backup.zpak --level 22

# Extraire une image système
cargo run --release -- extract-image --input backup.zpak --output projet_restauré/
```

### Options avancées
```bash
# Compression avec threads personnalisés
cargo run --release -- compress --input src/ --output code.zpp --threads 8 --level 15

# Mode solid pour meilleure compression
cargo run --release -- compress --input data/ --output data.zpp --solid --level 22
```

## 🏗️ Architecture

### Modules principaux
- **`compress.rs`** : Compression traditionnelle avec détection de types
- **`decompress.rs`** : Décompression avec validation d'intégrité
- **`image.rs`** : Système d'images avec déduplication par blocs
- **`profile.rs`** : Profils de compression par type de fichier
- **`error.rs`** : Gestion d'erreurs typée

### Format d'archive (.zpak)
1. **Header** : Version, métadonnées, statistiques
2. **Index des blocs** : Hash et position de chaque bloc unique
3. **Données compressées** : Blocs zstd dédupliqués
4. **Métadonnées fichiers** : Arborescence et références aux blocs

## 🧪 Tests

```bash
# Tests unitaires (dans les modules)
cargo test

# Tests avec verbose
cargo test -- --nocapture

# Exemple d'utilisation
cargo run --bin basic_usage
```

## 📈 Avantages vs concurrence

| Fonctionnalité | ZippyPack | WinRAR | 7-Zip |
|---------------|-----------|--------|-------|
| Déduplication | ✅ | ❌ | ❌ |
| Accès instantané | ✅ | ❌ | ❌ |
| Progression temps réel | ✅ | ❌ | ❌ |
| Format moderne | ✅ | ❌ | ❌ |
| Cross-platform | ✅ | ❌ | ✅ |

## 🔬 Cas d'usage optimaux

- **Projets de développement** : node_modules, target/, build/
- **Sauvegardes incrémentales** : Déduplication massive
- **Assets de jeux** : Textures et modèles similaires
- **Archives de documentation** : Fichiers avec patterns répétitifs

## 🛣️ Roadmap

- [ ] Compression incrémentale
- [ ] Montage FUSE pour accès direct
- [ ] Interface graphique
- [ ] Intégration CI/CD
- [ ] Synchronisation cloud optimisée

## 🤝 Contribution

Les contributions sont les bienvenues ! Consultez les [issues](https://github.com/Kamionn/zippypack/issues) pour les tâches en cours.

## 📄 Licence

Ce projet est sous licence MIT. Voir le fichier [LICENSE](LICENSE) pour plus de détails.

## 🏆 Benchmarks

```bash
# Générer des fichiers de test
rustc tools/generate_test_files.rs && ./generate_test_files

# Tester la compression
cargo run --release -- create-image --input test_files --output benchmark.zpak --level 22

# Comparer avec d'autres outils
# WinRAR: 268 KB
# 7-Zip: 324 KB  
# ZippyPack: 284 KB
```

---

**ZippyPack** : Parce que chaque byte compte. 🚀
