# Prompt — revue de code P2 de la story `58-1-numero-facture-textes-tests` (#1115, #1117, #1118, #1119, #1120, #1122, #1123, #1124), passe complète

Tu conduis, **en lecture seule**, la revue de code P2 de la story 58-1 de Kesh (comptabilité suisse en production ; Rust/Axum,
MariaDB, SvelteKit 5), avec le workflow BMAD `bmad-code-review`. Story de l'epic de nettoyage 58 : textes des refus de numéro de
facture, miroir TypeScript du format, tests ; **aucune migration, aucune règle métier, aucun changement d'API**.

- **Modèle : Sonnet** (la P1 était Opus : on alterne). **Contexte frais. Passe complète.** Rapport en français.
- Worktree : `/home/gcorbaz/devel/kesh-58-1`, branche `story/58-1-numero-facture-textes-tests`. Base de comparaison : `56d47b2cc`.
  La P1 a été jouée sur la tête `449fc5de6` (dernier commit de code `407440f0b`) ; la remédiation de la P1 suit. Relis **tout** le
  diff `56d47b2cc..HEAD`, et en particulier les commits postérieurs à `449fc5de6` (`git log 449fc5de6..HEAD`).
- Lis d'abord : le `CLAUDE.md` du worktree (*Review Iteration Rule*, *Le prompt d'une passe doit NOMMER le manuel*, *Une passe
  déclare ses axes exercés*, *Inventorier les sites NON RÉSOLUS*, *Propagation post-patch*, *Recompter*, *Recette visuelle*,
  *Priorités des défauts*) ; la fiche `_bmad-output/implementation-artifacts/58-1-numero-facture-textes-tests.md` (AC1 à AC9,
  Décisions D1 à D5, Journal des sorties, Dev Agent Record) ; le rapport de la P1,
  `/home/gcorbaz/devel/kesh-gate-logs/e58/58-1-revue-p1.md` (0/0/2/5), à lire en entier.
- Captures de la recette visuelle (exigées par R1-2) : `/home/gcorbaz/devel/kesh-gate-logs/s581/captures/` (neuf PNG, `01` à `09`,
  décrits au tableau « Recette visuelle — rattrapée le 2026-11-27 » de la fiche ; script `s581/captures-s581.mjs`). **Si le dossier est
  vide ou absent, R1-2 n'est pas remédié : dis-le comme finding MEDIUM.** Lis les images (outil Read) : le texte entier se lit-il ?

## Interdits

Tu n'écris **aucun fichier** hors de ton rapport. Tu ne lances **ni** `cargo build|clippy|fmt` hors de la commande permise ci-dessous,
**ni** `make` dans le dépôt, **ni** `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`,
`scripts/recette-montee.sh`, `scripts/install-hooks.sh`, **ni** `docker` / `docker compose`, **ni** `mariadb` / `sqlx migrate`,
**ni** `GRANT` / `REVOKE`, **ni** `git commit|checkout|stash|restore|reset|push|worktree`, **ni** `gh` en écriture, **ni**
`sed -i`, `tee`, `>` ou `>>` vers un fichier du dépôt, **ni** `pkill`, `killall`, `kill` par motif. Rien sous `~/kesh/`.
Permis : `git log|show|diff|grep`, `grep`, `sed -n`, `cat`, `gh … view|list` ; `pdftotext … -` ; une copie de `docs/manual` sous
`/tmp/claude-1000/` pour y lancer `make user` ; et, pour vérifier un finding :
`flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh cargo nextest run -E 'binary(<x>)'`
avec `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3306/kesh_e581`, et `scripts/mem-guard.sh npm --prefix frontend run test:unit -- <fichier>`.

Rapport : **`/home/gcorbaz/devel/kesh-gate-logs/e58/58-1-revue-p2.md`** (le seul fichier que tu écris).

## Axes (chacun déclaré exercé, ou non exercé avec sa raison — un « 0 finding » sans cette liste ne compte pas)

0. **Remédiation de la P1.** Pour R1-1 à R1-7 : remédié ou non (cite la ligne) ; la remédiation fait-elle naître un défaut neuf ?
   En particulier R1-1 : le conseil « ou le nom de l'exercice » est-il conditionné à `{FY}` dans `docs/manual/fr/user-manual.tex`
   (phrase du refus `NUMBER_FORMAT_UNRENDERABLE`) **et** `docs/api-external.md` (§ *Valider une facture — un numéro impossible à
   produire*) ? Greppe la valeur sur tout le dépôt (`ou le nom de l`, `or the fiscal year name`, `oder den Namen`, `oppure il nome`).
1. **Exactitude du code contre la fiche, AC1 à AC9**, critère par critère, ligne lue citée.
2. **Textes d'erreur des quatre locales** (`error-number-format-unrenderable-too-long`, `-duplicate`, `invoices-format-error-too-long`)
   et leur **repli Rust** (`crates/kesh-api/src/errors.rs`, garde G9 de `crates/kesh-api/tests/textes_coherents.rs`) : sens identique
   au fr-CH, échappement `{"{"}FY{"}"}` / `{{FY}}`.
3. **Miroir TypeScript contre Rust** (`frontend/src/lib/features/invoices/invoice-number-format.ts` contre
   `crates/kesh-core/src/invoice_format.rs`) : blancs (espace et tabulation seuls), longueur en **octets** UTF-8 jugée avant les
   caractères, `trim`, `{SEQ:+4}`, numéro interne (`{ numbering: true }`). **Un test qui compare deux sorties d'un même extracteur
   ne prouve rien** : vérifie que chaque côté est testé contre des littéraux.
4. **Mutations déclarées** (Dev Agent Record, journaux `kesh-gate-logs/e58/s581-mut-*.log`) : tuent-elles pour la bonne raison
   (assertion visée, non un effet de bord) ? Une mutation que tu soupçonnes, rejoue-la (binaire ciblé, base `kesh_e581`), restaure
   le fichier **et touche-le**.
5. **Le manuel**, `docs/manual/fr/*.tex`, contre le code, **PDF compris** : le PDF n'est pas régénéré dans la branche ; copie
   `docs/manual` sous `/tmp/claude-1000/`, `make user`, puis aplatis (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) et cherche
   chaque phrase changée (apostrophes typographiques `’` dans le PDF : greppe sans l'apostrophe).
6. **`docs/api-external.md`** (`NUMBER_FORMAT_UNRENDERABLE`, deux lieux) contre le code.
7. **Rectificatifs datés** (`e35-choix-autonomes-e351.md`, `35-1-numerotation-sans-seq.md`) : un relevé daté ne se réécrit pas, il
   s'annote ; chaque citation du rectificatif existe-t-elle à la ligne qu'il nomme (R1-4) ?
8. **Propagation des valeurs changées** sur tout le dépôt (code, doc-comments, tests, quatre catalogues, replis Svelte/TS, manuels,
   `docs/`) : greppe le jeton, non la formulation.
9. **Recette visuelle** : captures de chaque état touché (refus `too_long`, refus `duplicate`, page *Administration → Facturation* :
   espace insécable, 33 « é » → « 64 octets (actuel : 71) », numéro interne ; captures 01 à 09), thème clair. Le texte entier se lit-il dans l'avis ?

Pour tout finding CRITICAL ou HIGH qui affirme une absence ou un anti-pattern : preuve `grep -nF "<chaîne>" <fichier>` ou extrait
lu, cité au rapport. Classe chaque finding CRITICAL/HIGH/MEDIUM/LOW avec sa preuve, puis le triage (decision-needed, patch, defer,
dismiss) et la liste des axes exercés et non exercés.
