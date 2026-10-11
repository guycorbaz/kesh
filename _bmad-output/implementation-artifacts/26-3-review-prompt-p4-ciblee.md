# Prompt — passe ciblée P4 de la revue de code de la Story 26-3 (Haiku, une lentille)

Une seule lentille, **chasseur de régression de la remédiation P3**, braquée sur **le seul commit**
`8ed247fa` : `git -C /home/gcorbaz/devel/kesh-26-3 show 8ed247fa`. Contexte : rapport P3
`/home/gcorbaz/devel/kesh-gate-logs/26-3-review-p3-ciblee.md` ; fiche
`_bmad-output/implementation-artifacts/26-3-manuel-et-restauration-synology.md` (Change Log, entrée « Passe
ciblée P3 remédiée »). Langue : français. ⚠️ Lis les fichiers **à HEAD** (`sed -n`, `grep -nF`) pour toute
affirmation de présence ou d'absence.

## Interdits absolus

Lecture seule. N'exécute AUCUN script du dépôt (`scripts/prepare-release.sh`, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`, `scripts/synology/*.sh`), ni `cargo`, `npm`, `npx`, `make`, `latexmk`, `docker`.
Aucun `git` qui écrit, aucune commande `gh` qui écrit. N'écris AUCUN fichier hors de ton rapport (ni PDF
temporaire, ni fichier dans `kesh-gate-logs/` autre que le rapport) : lis les PDF par `pdftotext … -`
(sortie standard). Permis : `git show`, `git diff`, `git grep`, `grep`, `sed -n`, `cat`, `bash -n`.

## Axes

1. **G16, clause 7.5 élargie** (`crates/kesh-api/tests/configuration_transmise.rs`, cherche `alimentée par`) :
   la regex correspond-elle **exactement** au code de `scripts/synology/kesh-restore.sh` à HEAD (`LISTE=`,
   `IFS='|' read`, `done <<< "$LISTE"`) ? Une boucle alimentée autrement, ou un `LISTE` lu par un autre
   `--format`, passerait-elle ? La clause 7.1 et la 7.5 rougissent-elles ensemble ou séparément comme le dit
   le Change Log (mutations (a), (e), `done < /dev/null`) ?
2. **Doc-comment de G14** : dit-elle vrai contre `docs/manual/fr/admin-manual.tex` (`grep -nE "docker
   compose"`) — `down`, `--tail=100`, `--since=1h` ?
3. **Manuel utilisateur** (phrase « avec leur propre message ») et son **PDF aplati**
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) : vrai contre
   `frontend/src/lib/features/reconciliation/ManualMatchModal.svelte`, `TransactionSplitModal.svelte` et le
   catalogue `fr-CH` ?
4. La remédiation a-t-elle laissé un site du même symptôme ailleurs (G16 doc-comment, fiche) ?

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/26-3-review-p4-ciblee.md` : findings avec sévérité, fichier:ligne,
**sortie `grep -nF`/`sed -n` copiée** ; **axes exercés ET non exercés** ; bilan chiffré.
