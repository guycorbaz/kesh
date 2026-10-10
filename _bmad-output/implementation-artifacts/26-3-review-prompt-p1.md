# Prompt — revue de code P1 de la Story 26-3 (passe complète, trois lentilles)

Story : `_bmad-output/implementation-artifacts/26-3-manuel-et-restauration-synology.md` (worktree
`/home/gcorbaz/devel/kesh-26-3`, branche `story/26-3`). Issues #616, #617, #619. Langue : **français**.

**Périmètre** : le diff aplati `git -C /home/gcorbaz/devel/kesh-26-3 diff {BASE}..{HEAD}` — `{BASE}` est
le commit de clôture de la validation, `{HEAD}` le dernier commit du développement. Les PDF du diff se lisent
par `pdftotext <f> - | tr '\n' ' ' | tr -s ' '` (forme **aplatie** : `pdftotext` coupe les lignes, un
`grep` naïf sur une phrase rend un faux négatif). Pour les comptages sensibles aux apostrophes et
ligatures, la fonction `occ` de l'encadré en tête des AC de la fiche.

Trois lentilles, chacune dans un **contexte frais**, par un sous-agent distinct :

- **B — Blind Hunter** : lit le **diff seul** (pas la fiche, pas les issues). Cherche les défauts de code
  et de texte : erreurs shell (`set -euo pipefail`, sous-shell, `read`, `IFS`, `[[ =~ ]]`, guillemets),
  regex Rust fausses ou trop larges/étroites, contradictions dans les manuels, LaTeX cassé, i18n incohérente.
- **E — Edge Case Hunter** : lit le diff **et** le dépôt. Parcourt chaque branche et chaque frontière :
  sorties réelles de `docker ps --format` (étiquette absente, noms, images par empreinte/identifiant,
  états `removing`, lignes vides, sortie terminée par un saut de ligne), comportement du faux `docker` de
  la recette (portée de la fonction exportée, `set -u`, traps, ordre de la recette en `RECETTE_SECTION=0`
  et en mode complet), gardes G14/G16/G20 (ce qu'elles ne voient pas, faux positifs, tests muets).
- **A — Acceptance Auditor** : lit la **fiche** (12 AC) et le diff ; vérifie chaque AC point par point,
  y compris les décomptes et les « attendus » de l'AC 11.2 (rejouer les commandes bornées, **code de
  sortie compris** : 0 ou 1, jamais 128), et le Dev Agent Record (ne déclare-t-il que ce qui a tourné ?).

## Axes obligatoires, pour les trois lentilles

1. **Le manuel** — `docs/manual/fr/user-manual.tex`, `docs/manual/fr/admin-manual.tex` et leurs **PDF**
   (`docs/manual/fr/*.pdf`, contrôlés **aplatis**) : chaque phrase neuve est-elle vraie **contre le code** ?
   Écrans : `frontend/src/routes/(app)/accounts/+page.svelte`, `bank-accounts/+page.svelte`,
   `bank-import/+page.svelte`, `frontend/src/lib/features/bank-import/*.svelte`, catalogues
   `crates/kesh-i18n/locales/*/messages.ftl`, `crates/kesh-api/src/routes/bank_imports.rs`,
   `reconciliation.rs`. Scripts : `scripts/synology/kesh-restore.sh`, `kesh-dump.sh`. Le manuel promet-il
   une protection absente, ou omet-il un chemin ? *(C'est en vérifiant le manuel que les défauts les plus
   chers ont été trouvés sur l'Epic 24.)*
2. **La garde de `kesh-restore.sh`** (étape 1) : est-elle fermée dans tous les cas ? Existe-t-il une
   sortie de `docker ps` réelle qui la fasse passer sous un Kesh actif, ou refuser à tort un hôte sain ?
   L'ordre des étapes, `$1` employé une seule fois, aucune commande `docker compose` exécutée.
3. **La section 0 de la recette** (`scripts/synology/recette.sh`) : prouve-t-elle ce qu'elle annonce ?
   Le faux `docker` peut-il fuir hors des sous-shells ? Un cas peut-il être vert à vide ?
4. **G14, G16, G20** (`crates/kesh-api/tests/configuration_transmise.rs`, `textes_coherents.rs`) :
   les regex tiennent-elles la forme écrite ; une écriture fautive passerait-elle ?
5. **Propagation** : le symptôme corrigé (« Compte de banque », « toutes les banques », « Import bancaire
   CAMT.053 », « Nouvel import », `service=kesh-api`) subsiste-t-il ailleurs — `docs/`, `website/`,
   `README.md`, catalogues des quatre locales, replis Svelte, `CHANGELOG.md` (une seule section
   `## [0.13.1] — Non publié`) ?

## Interdits absolus

Lecture seule. **N'exécute aucun script du dépôt** : ni `scripts/prepare-release.sh`, ni
`scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/synology/*.sh` (la recette crée des
conteneurs) ; ni `cargo`, `npm`, `npx`, `make`, `latexmk`, `docker`. Aucun `git` qui écrit (`commit`,
`checkout`, `reset`, `stash`, `tag`, `push`), aucune commande `gh` qui écrit. Permis : `git show`,
`git diff`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `wc`, `pdftotext … -`, `bash -n` (syntaxe
seule). N'écris **rien** hors de ton rapport.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/26-3-review-p1-<B|E|A>.md` (selon la lentille) :

- chaque finding : sévérité (CRITICAL / HIGH / MEDIUM / LOW), fichier:ligne, description, **preuve copiée**
  (sortie `grep -nF` / `sed -n` / `pdftotext` aplati) — toute affirmation de présence ou d'absence porte
  sa sortie `grep -nF` ; correction proposée ;
- **la liste des axes exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne
  compte pas comme passe ;
- un bilan chiffré (n CRITICAL / n HIGH / n MEDIUM / n LOW).
