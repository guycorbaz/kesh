# Manuels Kesh

Documentation officielle Kesh au format LaTeX → PDF, organisée en trois manuels :

| Manuel | Fichier source FR | Public cible | Description |
|--------|-------------------|--------------|-------------|
| **Manuel administrateur** | `fr/admin-manual.tex` | Administrateurs système / DevOps PME | Installation, configuration, déploiement, sauvegarde, mise à jour, conformité OLICo |
| **Manuel utilisateur** | `fr/user-manual.tex` | Utilisateurs finaux (PME, fiduciaires, comptables) | Toutes les fonctionnalités Kesh : auth, plan comptable, écritures, factures QR Bill, import bancaire, rapports, exports |
| **Brochure marketing** | `fr/marketing-brochure.tex` | Prospects, partenaires, presse | Présentation commerciale courte, différenciateurs, cas d'usage, call-to-action |

## Structure du répertoire

```
docs/manual/
├── README.md                    # Ce fichier
├── Makefile                     # Build via xelatex
├── shared/                      # Assets partagés
│   ├── kesh-style.sty           # Style LaTeX commun (couleurs, polices, boîtes)
│   └── kesh-logo.svg            # Logo Kesh (copie depuis website/img/logo.svg)
├── fr/                          # Sources canoniques (français)
│   ├── admin-manual.tex
│   ├── user-manual.tex
│   └── marketing-brochure.tex
├── de/                          # Traductions allemandes (à compléter — voir README.md)
├── it/                          # Traductions italiennes (à compléter)
└── en/                          # Traductions anglaises (à compléter)
```

## Build

### Prérequis

- **xelatex** (TeX Live ≥ 2021 recommandé)
- **Polices** : TeX Gyre Heros (inclus TeX Live par défaut). Optionnel pour un rendu plus moderne : Inter + JetBrains Mono.

Sur Debian/Ubuntu :
```bash
sudo apt install texlive-xetex texlive-latex-extra texlive-fonts-extra fonts-inter fonts-jetbrains-mono
```

Sur macOS (Homebrew) :
```bash
brew install --cask mactex
brew install --cask font-inter font-jetbrains-mono
```

### Compilation

```bash
# Tous les PDF FR (canoniques)
make

# Manuel administrateur seul
make admin

# Manuel utilisateur seul
make user

# Brochure marketing seule
make brochure

# Toutes les langues disponibles (FR + traductions présentes)
make all-langs

# Nettoyer les fichiers temporaires LaTeX
make clean

# Nettoyer tout (y compris les PDF)
make distclean
```

Les PDF sont générés dans le même répertoire que les sources `.tex`. Par exemple :
- `fr/admin-manual.tex` → `fr/admin-manual.pdf`

## Style et look

Les manuels utilisent un **style moderne**, pas le défaut LaTeX (Computer Modern) :

- **Polices** : Inter (si installée) ou TeX Gyre Heros (fallback) — sans-serif moderne style Helvetica.
- **Couleurs brand** : palette cohérente avec le design system frontend Kesh (`#1e40af` primary blue, `#3b82f6` light, `#16a34a` success, etc.).
- **Mise en page** : marges aérées (2.5–2.8 cm), interligne 1.5, paragraphes espacés sans indentation.
- **Boîtes d'information** : `\keshnote{}`, `\keshtip{}`, `\keshwarning{}`, `\keshdanger{}` (style filets colorés à gauche, fond légèrement teinté).
- **Code source** : fond sombre `#0F172A`, texte clair, coloration syntaxique via `listings`.
- **Tableaux** : style `booktabs` (pas de bordures lourdes, en-têtes colorés).
- **Liens hypertexte** : couleur brand `keshPrimary`, sans cadre.

Le fichier `shared/kesh-style.sty` centralise toute la configuration. Modifier ce fichier propage le changement à tous les manuels.

## Mise à jour à chaque release

> ⚠️ **Important** : les manuels Kesh sont versionnés et doivent être mis à jour **à chaque release** du logiciel.

### Process de mise à jour à une release `vX.Y.Z`

1. **Mettre à jour les variables produit** dans `shared/kesh-style.sty` :

   ```latex
   \providecommand{\keshVersion}{X.Y.Z}
   \providecommand{\keshReleaseDate}{YYYY-MM-DD}
   \providecommand{\keshTargetRelease}{vX.Y}
   ```

2. **Mettre à jour le contenu des manuels** pour refléter les nouvelles fonctionnalités, changements UI, breaking changes, etc. :
   - **Manuel administrateur** : nouvelles env vars, nouvelles migrations DB, changements docker-compose, nouvelles dépendances.
   - **Manuel utilisateur** : nouvelles features, nouvelles routes UI, nouveaux workflows, captures écran à mettre à jour.
   - **Brochure marketing** : nouveau pitch si positionnement évolue, mise à jour de la roadmap visible, nouveaux différenciateurs.

3. **Rebuild les PDF** :

   ```bash
   make distclean && make all-langs
   ```

4. **Inclure les PDF dans le tag release GitHub** :
   - Les PDF générés (`fr/*.pdf`, `de/*.pdf`, etc.) sont attachés au release GitHub `vX.Y.Z`.
   - Idéalement automatisé via `.github/workflows/release.yml` (à implémenter dans Epic 10 « Déploiement & Opérations »).

5. **Vérifier la cohérence** entre :
   - Le contenu des manuels.
   - Le `README.md` du repo (feuille de route).
   - Le `CHANGELOG.md` (si présent).
   - Le sprint-status.yaml (sources de vérité projet).

### Process de release automatisé (futur Epic 10)

L'objectif Epic 10 est d'automatiser cette mise à jour via un workflow GitHub Actions déclenché par les tags `v*.*.*` :

```yaml
# .github/workflows/release.yml (extrait conceptuel)
on:
  push:
    tags: ['v*.*.*']
jobs:
  build-manuals:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: sudo apt-get install texlive-xetex texlive-latex-extra fonts-inter fonts-jetbrains-mono
      - run: cd docs/manual && make all-langs
      - uses: softprops/action-gh-release@v2
        with:
          files: docs/manual/*/*.pdf
```

Cette automatisation est suivie dans l'issue GitHub Epic 10 (à créer pré-kickoff).

## Traductions DE/IT/EN

Les traductions allemande, italienne et anglaise sont **à compléter** dans des stories séparées (Epic 10+ ou Epic dédié). La source canonique est le français (FR).

Pour ajouter une traduction d'un manuel :

1. Copier le `.tex` depuis `fr/` vers le dossier langue cible :
   ```bash
   cp fr/admin-manual.tex de/admin-manual.tex
   ```
2. Adapter le préambule du fichier copié pour charger le bon babel :
   ```latex
   \usepackage[ngerman]{babel}  % DE
   % \usepackage[italian]{babel}  % IT
   % \usepackage[english]{babel}  % EN
   ```
3. Traduire intégralement le contenu (titres, sections, paragraphes, listes, boîtes d'information, captions tableaux).
4. Conserver les références techniques inchangées (noms de variables, chemins de fichiers, commandes shell, snippets de code).
5. Lancer `make all-langs` pour vérifier la compilation.
6. Faire reviewer par un native speaker idéalement (référence : Story 9-1 L4 « Traductions DE/IT/EN review par native speakers v0.2 »).

## Licence

Les manuels Kesh sont distribués sous la même licence que le logiciel Kesh : **[EUPL 1.2](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12)**.

## Contribuer

Pour signaler une erreur ou suggérer une amélioration des manuels :

- **Erreur factuelle ou incohérence avec le produit** : ouvrir une issue GitHub avec le template `bug_report.yml`.
- **Amélioration éditoriale ou nouvelle section** : ouvrir une issue avec `feature_request.yml`.
- **Traduction d'un manuel** : ouvrir une PR avec le contenu traduit + référencer l'issue de tracking de la traduction.

Toute issue / PR doit référencer la version Kesh ciblée et indiquer si elle doit attendre la prochaine release ou être publiée en hotfix de documentation.
