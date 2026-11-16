# Prompt — revue de code P2 de la Story 36-18a, passe **ciblée** (une lentille, sur la remédiation de la P1)

Tu relis **la seule remédiation de la revue P1** de la Story 36-18a de Kesh (comptabilité suisse, SvelteKit 5 runes),
release 0.18.0 : les montants « 1.250 » des soldes de départ, de la facture fournisseur (prix, montant attendu) et du
règlement, refusés comme ambigus. La P1 a rendu 0 CRITICAL, 0 HIGH, 1 MEDIUM, dix-huit LOW (rapports
`/home/gcorbaz/devel/kesh-gate-logs/e36-18/review-p1-{A,B,E}.md`). Le motif connu (`CLAUDE.md`, § *La passe ciblée*) : la
sévérité se déplace vers ce qu'on vient d'écrire. **C'est la remédiation qu'il faut relire, pas la story.**

Worktree `/home/gcorbaz/devel/kesh-36-18`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-36-18 diff ac953dea2 24c812dca` (`c3b26815b` remédiation, `24c812dca` titre d'un test). Les
commits suivants ne portent que la fiche, le registre et ce prompt. Modèle : **Haiku, Sonnet ou Opus** (passe ciblée) ;
contexte frais ; rapport en français.

Lis : ce diff ; dans la fiche `_bmad-output/implementation-artifacts/36-18a-montants-ambigus-livres.md`, le Dev Agent Record
§ *Remédiation de la revue P1* et l'entrée du Change Log du 2026-11-16 « Remédiation » ; au registre
`e36-choix-autonomes-36-18.md`, C-36-18-13 et C-36-18-14 ; les rapports P1 (pour savoir ce qui était demandé). Journaux (lecture
seule) : `kesh-gate-logs/e36-18/rem1-test-fast.log`, `rem1-front.log`, `rem1-e2e-spec.log`, `rem1-vitest-cible.log`,
`mut/resume.log`, `mut/M19.log` à `mut/M25.log` ; captures reprises `kesh-gate-logs/e36-18/captures/e3618a-*.png` (quatorze,
dont `e3618a-soldes-1250-ecriture.png` neuve).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris aucun fichier hormis ton rapport. **Aucun fichier temporaire**, **aucune redirection** (`>`,
  `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se déclare.
- **Aucun `rm`**, ni `pkill`, `killall`, `kill`. **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Rien qui écrit, compile ou exécute** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq`, ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, ni aucun script du
  dépôt ou de `kesh-gate-logs`, ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `curl`, **`claude`**, `sed -i`,
  `playwright`, `vitest`. **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, et pour le PDF la seule forme
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`. Toute affirmation de présence ou
  d'absence s'appuie sur une sortie `grep -nF` ou `sed -n` copiée ; tout chemin cité se vérifie.

## La lentille : chasseur de régressions de la remédiation

Pour chaque hunk du diff, demande-toi ce qu'il **casse** ou laisse **à vide** :

1. **Le scan QR** (`SupplierInvoiceForm.svelte`, `onScanFile`) : `readableUnambiguousAmount(p.expectedAmount, 2)` sur un
   montant vide, non numérique, à zéro décimale (`1250`), à deux (`1250.00`), à trois (`1.250`, `12.500`), à quatre
   décimales ; un montant QR EUR. Le montant pré-rempli diffère-t-il désormais du montant du QR au point de fausser le
   contrôle `amountMismatch` ou le refus serveur `AMOUNT_MISMATCH` ? Les deux tests du scan mènent-ils vraiment
   `onScanFile` (fichier, `decodeQrFromImageFile`, `scanQrSupplierInvoice`, `scanToPrefill`) ou passent-ils à vide ?
2. **Le refus effacé à la saisie** (`oninput={() => (formError = '')}` sur le prix et le montant attendu) : coexistence avec
   `bind:value` (ordre des mises à jour en Svelte 5) ; un refus **serveur** (`describeSubmitError`, `formError`) effacé à tort ?
   les autres champs (description, quantité, compte) laissent-ils le bandeau périmé — incohérence nouvelle ?
3. **Les soldes rognés** (`openingAmountReader`, `rowProblem`, `payloadOf`, `isEnteredLine`) : un site de la page ou de
   `opening-balances-totals.ts` qui lit encore la saisie brute (`git grep -n "journalAmountReader\|journalAmountPayload\|lineAmountProblem" -- 'frontend/src/routes/(app)/settings/opening-balances' frontend/src/lib/features/opening-balances`) ;
   `aria-invalid` (`unread`) et message d'accord ; une ligne de blancs seuls ; l'exclusivité débit/crédit (`onDebitInput`
   teste `row.debit !== ''` : un blanc vide-t-il l'autre côté ?) ; `retainedEarningsWarning` et `complementCounterpart`.
4. **La contrepartie tue** (`{#if complementInvalid}`, `data-side="invalid"`) : un test existant qui lisait `data-side` change-t-il
   de sens ? le rendu d'une ligne valide plus une ligne non lue ?
5. **Les tests neufs** (M19 à M25) : chacun rougit-il pour la raison qu'il annonce ? un titre qui ne dit pas ce que le test
   fait (le défaut d'E6, relevé en P1) ? la mutation qui le vise (`mut/M*.log`) est-elle tuée par lui ?
6. **Le manuel** (`docs/manual/fr/user-manual.tex`) : le paragraphe *Le montant saisi* du règlement et la phrase « un espace
   avant ou après le montant … est ignoré » des soldes sont-ils vrais au code ? le paragraphe *Enregistrer* est-il redevenu
   continu ? le PDF n'est pas régénéré (à l'intégration) : point d'intégration, non défaut.
7. **Les comptes rendus de la remédiation** (fiche, registre, commentaires des compteurs) : chaque chiffre se retrouve-t-il au
   journal cité, à la tête `24c812dca` ? les décomptes (dix-neuf findings, seize distincts, quatorze corrigés) recomptés depuis
   les trois rapports P1 ? C-36-18-13 et C-36-18-14 disent-ils vrai du code cité ?

## Rapport

Un fichier : `/home/gcorbaz/devel/kesh-gate-logs/e36-18/review-p2-R.md`. Findings numérotés (`R1`…) avec sévérité
(CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue copiée, preuve, remède ; **la liste des axes exercés ET
non exercés** ; le décompte final recompté depuis la liste. Un « 0 finding » non adossé à cette liste ne compte pas.
