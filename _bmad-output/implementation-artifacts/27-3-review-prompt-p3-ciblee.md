# Prompt — revue de code P3 ciblée, Story 27-3 (une lentille, Haiku)

*Versionné le 2026-10-12, par l'agent de remédiation de la revue de code P2 ciblée. Contexte frais, lecture seule.*

Worktree `/home/gcorbaz/devel/kesh-27-3`, branche `story/27-3-profil-apres-apercu`. **Le seul diff à relire**, aplati
(un seul diff, pas la suite des commits — CLAUDE.md, § *Haiku-specific guardrails*) :

```
git diff f8241d38 5632a2bf -- . ':(exclude)_bmad-output'
```

`f8241d38` = état revu par la P2 ; `5632a2bf` = **seul commit de code** de la remédiation P2 (4 fichiers). Le commit
suivant, `0e5b4d54`, ne porte que le manuel (`.tex` + PDF) : relis-le avec `git diff 5632a2bf 0e5b4d54 --
docs/manual/fr/user-manual.tex` pour l'axe 4 seulement. Les commits ultérieurs ne touchent que `_bmad-output/`.
⚠️ Pathspec `':(exclude)…'`, et **non** `':!…'`. Fiche : `_bmad-output/implementation-artifacts/27-3-profil-apres-apercu.md`
(Change Log, section *Revue de code P2 ciblée*) ; registre : `_bmad-output/implementation-artifacts/epic-27-choix-autonomes-27-3.md`
(C-27-3-57) ; rapport de la P2 : `/home/gcorbaz/devel/kesh-gate-logs/27-3-review-p2-ciblee.md` ; journal des mutations :
`/home/gcorbaz/devel/kesh-gate-logs/27-3-mutations.log` (lignes M36 à M40 et le commentaire qui les décrit).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de `kesh-gate-logs/` (rien sous
`~/kesh-test/`).

## Ce que la remédiation a changé (à vérifier, pas à croire)

- **Production** — `frontend/src/lib/features/bank-import/BankImportUpload.svelte`, fonction `loadProfiles()` et une
  variable `profilesSeq` : (M1) après une relecture réussie, un `bankProfileId` absent de la liste relue revient à
  `null` ; (L4) un numéro de séquence fait qu'une relecture dépassée par une plus récente ne pose rien. C'est la
  **seule** modification de code de production.
- **Tests** — `BankImportUpload.test.ts` : V6-quater (profil supprimé), V6-quater (liste vide), V6-quater (séquence) ;
  V1 et V14 comptent les appels de `listBankProfiles` (L1). `bank-import-page.test.ts` : texte exact d'un paragraphe (L2).
  `frontend/tests/e2e/bank-csv-import.spec.ts` : `ubsFixtureIn` marque le libellé `Facture …-014` du passage (`RUN`) et
  lève si la marque manque (L3).

## Axes (chacun déclaré exercé ou non)

1. **La remise à `null` et ses voisins.** Lis `loadProfiles`, `runPreview`, `handleProfileChange`, `handleConfirm` et
   `reset` du fichier final. La remise à `null` peut-elle effacer un profil que l'utilisateur vient de choisir (geste
   sur le sélecteur pendant la relecture) ? Peut-elle survenir au montage ? La présélection auto-détectée (`runPreview`,
   `profileId === null && autoMatched`) peut-elle poser, après la remise à `null`, un id absent de la liste ? Le recalcul
   en vol part-il toujours avec le profil de l'instantané (et non `bankProfileId`) ? Le chemin de sortie décrit
   (« redéposez le fichier ») est-il atteignable : zone de dépôt affichée après l'échec du recalcul ?
2. **La garde de séquence.** `profilesSeq` est-elle une variable ordinaire (pas un `$state`, cf. `seq`) ? Un échec
   (`catch`) d'une relecture dépassée peut-il produire un effet ? Une relecture du montage peut-elle être dépassée par
   celle d'un `409`, et est-ce correct ?
3. **Les tests neufs prouvent-ils ce qu'ils nomment ?** Pour chaque V6-quater, le montage construit-il l'état voulu
   (`relistWith` posé avant `staleConfirm` : consommé par la relecture du `409` et non par l'`onMount`, déjà passé) ?
   Les mutations M36 à M40 rougissent-elles **pour la raison nommée** ? Cherche une **mutation plausible du diff qui
   resterait verte** (ex. : remise à `null` placée avant la garde de séquence ; comparaison `==` contre un id chaîne ;
   relecture qui remet à `null` même quand le profil est présent ; la marque `RUN` de l'E2E qui casserait le parse ou
   un compte de transactions attendu).
4. ⛔ **LE MANUEL — `docs/manual/fr/user-manual.tex` ET `user-manual.pdf`.** La phrase neuve (« Un profil supprimé
   disparaît du sélecteur, qui revient sur Auto-détection ; pour l'auto-détection, redéposez le fichier — c'est aussi la
   marche à suivre si plus aucun profil n'existe, le sélecteur n'étant alors plus affiché ») est-elle vraie au code, et
   ne contredit-elle pas la phrase qui précède (« choisissez un autre profil ») ni l'encadré du haut (« inutile de
   redéposer le fichier pour essayer un autre profil ») ? Contrôle le **PDF aplati**
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/home/gcorbaz/devel/kesh-gate-logs/27-3-review-p3-ciblee-manuel.txt` ; l'apostrophe y est typographique, écrire `.`
   à sa place dans un motif).
5. **Propagation** : le comportement corrigé (« choisissez un autre profil », profil supprimé, `loadProfiles`) est-il
   décrit ailleurs de façon devenue fausse — `CHANGELOG.md`, `docs/api-external.md`, fiche, registre ? (`grep -rnF`,
   en excluant `node_modules` et `target`.)

⚠️ **Erreur de catégorie connue** : un livrable que la fiche prescrit mais qui n'est pas dans ce diff n'est pas un
défaut de la remédiation. Toute affirmation de présence ou d'absence d'un code ou d'un texte s'appuie sur une sortie
`grep -nF` **copiée**, prise sur le fichier final (`grep -nF "<chaîne>" <fichier>`, `-A <N>` pour un bloc) ; un finding
CRITICAL ou HIGH sans elle sera écarté.

## Interdits

Lecture seule. Tu n'écris que `/home/gcorbaz/devel/kesh-gate-logs/27-3-review-p3-ciblee.md` (et le texte aplati du PDF
nommé à l'axe 4). ⛔ **Aucune commande qui compile, exécute ou écrit** : ni `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), ni `make`, `latexmk`, `xelatex`, ni
`cargo` (aucune sous-commande), `sqlx`, `npm`, `npx`, `node`, ni **aucun interpréteur** — `python`, `python3`, `perl`,
`ruby`, `awk` avec un programme, `bash -c`, `sh -c` —, ni `docker`, ni `git` en écriture (`commit`, `add`, `checkout`,
`switch`, `stash`, `reset`, `rebase`, `apply`), ni `gh` en écriture, ni SQL, ni HTTP. Autorisés : lecture (`Read`),
`grep`, `sed -n`, `wc`, `git log` / `show` / `diff`, `pdftotext` vers `kesh-gate-logs/`.

## Rapport

Dans `/home/gcorbaz/devel/kesh-gate-logs/27-3-review-p3-ciblee.md` : décompte par sévérité (CRITICAL / HIGH / MEDIUM /
LOW) ; chaque finding avec `fichier:ligne`, **preuve** (commande et sortie copiées, ou code relu cité), correction
proposée ; ⛔ **la liste des axes exercés ET non exercés** (les cinq ci-dessus) — un « 0 finding » sans elle ne compte
pas comme passe.
