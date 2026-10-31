# Prompt — validation P2 de la fiche v017-3a (#671 : rattacher une transaction à une écriture existante), deux lentilles, Opus

Tu valides la **fiche** de la Story v017-3a de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11). **#671
(P2)** : une transaction bancaire déjà comptabilisée par un autre chemin que la réconciliation — facture fournisseur
payée depuis sa fiche, écriture saisie au journal, règlement par compte interne — reste en attente pour toujours. La
fiche retient un **rattachement à une écriture existante** dont le mouvement sur le compte du grand livre de la banque
égale la transaction (routes `entry-candidates` / `link-entry`, généralisation de la Story 29-8), un **avertissement**
« semble déjà comptabilisée » sur la page de propositions et une **confirmation** avant d'affecter une telle ligne ;
aucune migration, aucune donnée existante réécrite.

Worktree `/home/gcorbaz/devel/kesh-0173`, branche `story/v017-3-rapprochement-attente`, partie de `c5101b07`. Langue du
rapport : **français**. Modèle : **Opus**. Passe **complète** (deuxième passe de validation). **Contexte frais** : ne
t'appuie sur aucune passe antérieure pour juger, ni sur ce que la fiche dit d'elle-même.

La passe P1 (Sonnet) a rendu 0/1/12/12 ; sa remédiation est le commit `docs(v017-3a) : remédiation de la validation P1`
(`git log --oneline c5101b07..HEAD`, puis `git show <commit>`). Les rapports P1, à lire **après** ton propre examen et pour
l'axe obligatoire seulement : `/home/gcorbaz/devel/kesh-gate-logs/v017-3a-validate-p1-R.md` et `-F.md`.

## À lire

1. La fiche `_bmad-output/implementation-artifacts/v017-3a-rattacher-ecriture-existante.md`, **en entier**.
2. Le registre `_bmad-output/implementation-artifacts/v017-3-choix-autonomes.md` (C-v017-3-1 à 12, 17, 21 à 23).
3. Les issues, **en lecture seule** : `gh issue view 671 --json title,body,comments`, idem 672, 670, 418, 526, 687.
4. La contrainte du Project Lead : `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`.
5. `crates/kesh-db/src/repositories/settlement_cancellation.rs`, `crates/kesh-api/tests/reconciliation_link_e2e.rs`
   (t16, t17, t17bis), `crates/kesh-api/tests/textes_coherents.rs` (G9), `docs/manual/fr/admin-manual.tex:2966`. Puis la
   fiche et le code de la Story 29-8 dont celle-ci est la généralisation :
   `_bmad-output/implementation-artifacts/29-8-transaction-reglee-ailleurs.md` ;
   `crates/kesh-db/src/repositories/reconciliation_link.rs`, `reconciliation_cancel.rs`, `journal_entries.rs`
   (`reversal_blockers`, `reverse_in_tx`, `reverse_owned_in_tx`), `supplier_invoices.rs` (`pay_in_tx_as`),
   `payment_batches.rs` (`confirm_batch`), `bank_accounts.rs` (`ensure_open_in_tx`, `BALANCES_SQL`) ;
   `crates/kesh-api/src/routes/reconciliation.rs` (`get_proposals`, `link_preflight`, `get_link_candidates`,
   `post_link_transaction`, `link_once`) ; `crates/kesh-api/tests/audit_route_registry.rs`,
   `bank_account_closed_inventory.rs` ; `frontend/src/lib/features/reconciliation/` (`ReconciliationProposals.svelte`,
   `LinkSettlementModal.svelte`, `reconciliation.api.ts`, `reconciliation.types.ts`).
6. Le **manuel** : `docs/manual/fr/user-manual.tex`, § *Réconciliation bancaire* (l. 2290-2420 environ, dont
   `sec:rattacher-reglement`) et l. 3320-3340 ; le PDF `docs/manual/fr/user-manual.pdf`, aplati par
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, **vers la sortie standard seulement**.
7. La fiche parallèle, pour le seul point des collisions :
   `/home/gcorbaz/devel/kesh-0162/_bmad-output/implementation-artifacts/v016-2-releve-manuel.md` (lecture seule ;
   n'y écris rien).
8. `CLAUDE.md` du worktree (*Review Iteration Rule*, *Un appariement automatique propose, il ne crée jamais*, *Pattern
   batch*, *Migration breaking policy*, *Issue Tracking Rule*) et `docs/agents-orchestration.md`.

## Deux lentilles, en contexte frais chacune

### Axe obligatoire des deux lentilles — la remédiation P1

Le diff de la remédiation est-il juste ? Pour chaque finding P1 : appliqué comme le Change Log l'annonce, prémisse vraie
au code, et la remédiation a-t-elle **introduit** un défaut ? En particulier : `cancelBlockedBy` (scission de
`cancel_blocker` en `entry_cancel_blocker` : le classement et la précédence restent-ils une seule définition ? un motif
annoncé peut-il différer de celui de l'annulation réelle ?) ; `matchedElsewhereCount` (le second mode de la requête
est-il juste pour un virement, faux positif pour une écriture qui touche deux comptes bancaires sans être un virement ?) ;
les courses T-race-1 à 3 et M20 (exécutables sur le patron cité ?) ; le protocole de mesure de l'AC-11 ; les huit passages
du manuel (la liste est-elle maintenant close ? cherche au `.tex` **et** au PDF aplati, fragments courts, deux
apostrophes) ; le texte de l'avertissement et `{count}` ; les nouvelles mutations M2bis, M17 à M24 ; les collisions
(v016-2, v017-3b, manuel d'administration). Le symptôme corrigé a-t-il été grepé ailleurs (propagation) ? Les listes
« issues à ouvrir » et « LOW non pris » sont-elles fondées ?

L'orchestrateur lance **deux agents distincts** sur ce même prompt ; chacun ne tient **que** sa lentille.

### Lentille R — la règle, les verrous et leur preuve

- **Les prémisses** : chaque `fichier:ligne` cité par la fiche et le registre dit-il ce que la fiche lui fait dire ?
- **La règle (conditions 1 à 6)** : est-elle juste et **close** ? Cherche une écriture qui passerait à tort (écriture
  d'un exercice, d'ouverture, de clôture ; écriture d'une facture client validée ; écriture d'un rapprochement annulé ;
  écriture qui touche deux comptes bancaires ; écriture dont les lignes sur le compte de la banque se compensent) et une
  qui serait écartée à tort (règlement fournisseur par compte interne, règlement client par compte interne). Le signe
  est-il juste pour un débit et un crédit ? L'exclusion du domaine de la 29-8 (condition 6) laisse-t-elle un trou entre
  les deux listes, ou un recouvrement ?
- **L'invariant « une écriture, une transaction »** : le verrou de ligne de `journal_entries` le tient-il vraiment ?
  Établis, au code, les deux points (a) et (b) de l'AC-3 (contre-passation concurrente ; un geste qui tiendrait
  `journal_entries` puis `bank_transactions`). Un chemin qui pose `matched_entry_id` sur une écriture existante a-t-il
  été oublié ? Le virement entre deux comptes propres (L2) est-il correctement décrit ?
- **L'annulation** (C-v017-3-9, AC-8.3) : que fait réellement `reconciliation_cancel` sur une écriture saisie au journal
  rattachée, sur un règlement fournisseur rattaché, sur un règlement client par compte interne rattaché ? Une
  écriture possédée par un document (motif `OWNED_BY_*`) rend-elle l'annulation impossible, et la fiche le dit-elle ?
- **La proposition (AC-10, AC-11)** : `alreadyBooked` est-il calculé par les **mêmes** fonctions que les listes ? Le coût
  par page (jusqu'à 500 transactions) est-il raisonnable ; la requête sans fenêtre de dates somme-t-elle tout
  l'historique du compte à chaque appel ?
- **Les registres (AC-9)** : les valeurs de départ citées sont-elles exactes ? La forme d'inscription au `REGISTRE` et à
  `FENETRE` est-elle celle de `link_in_tx` ?
- **Les mutations (M1 à M16)** : chacune rougit-elle le test annoncé, pour la raison annoncée ? Une régression probable
  reste-t-elle sans mutation ? M10 survivante assumée est-elle justifiée ?
- **Les données existantes (AC-21, AC-22)** : une base 0.16.x (transactions rejetées selon la consigne d'attente) est-elle
  traitée sans réécriture ? Le chemin *Annuler le rejet* puis rattacher existe-t-il au code ?

### Lentille F — l'écran, les textes et ce qu'ils affirment

- **Le manuel (AC-18)** — axe obligatoire, au `.tex` **et** au PDF aplati : chaque phrase devenue fausse est-elle
  nommée ? La liste *Ce qui ne se rattache pas* récrite est-elle exacte au regard de la règle ? Le texte annonce-t-il
  avant le geste ce que fait l'annulation (contre-passation d'une écriture saisie) ? Une phrase d'une **autre** section
  (préparation de clôture, clôture d'un compte bancaire, factures fournisseur, paiements pain.001) devient-elle fausse ?
  Cherche « en attente », « rejetez », « Rattacher », « ne se rattache », « déjà comptabilis ».
- **L'écran (AC-12 à AC-16)** : l'avertissement dit-il vrai (« du même montant, non rapprochée(s) ») au regard de la
  règle ? La confirmation couvre-t-elle **tous** les gestes qui passent une écriture (accepter une facture client, une
  facture fournisseur, une règle ; affecter ; éclater) ? Le renommage du bouton casse-t-il des tests ou des E2E
  existants (`grep -rn "Rattacher à un règlement\|link-settlement-button" frontend`) ? Les captures prescrites
  couvrent-elles chaque état touché ?
- **i18n** : chaque texte visible nouveau ou récrit a-t-il sa clé dans les quatre catalogues ? Le message d'erreur et
  son repli d'`errors.rs` sont-ils prescrits égaux (garde G9) ?
- **`docs/api-external.md`, CHANGELOG (AC-19, AC-20)** : affirmations vraies au code ? Aucune version bumpée, aucune
  migration ?
- **Les collisions avec la v016-2** : la liste de la fiche est-elle complète (cherche les fichiers que les deux fiches
  prescrivent de toucher) ? Le point sur `audit_route_registry.rs` est-il exact ?
- **Les règles du dépôt** : aucun mot-clé de fermeture prescrit dans un message de commit ; données réelles absentes ;
  périmètre (la règle de découpage : la fiche touche-t-elle plus de cinq modules métier ?) ; défaut voisin non mécanique
  renvoyé à une issue ; « un appariement propose, il ne crée jamais » respecté partout.
- **Les décomptes** de la fiche et du registre (valeurs des registres, numéros de ligne, nombre de mutations) :
  recompte-les depuis la source.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, au chemin de ta lentille :
`/home/gcorbaz/devel/kesh-gate-logs/v017-3a-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/v017-3a-validate-p2-F.md` (lentille F). Il contient :

1. **Les findings**, numérotés `R1`, `R2`… ou `F1`, `F2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`),
   le lieu (`fichier:ligne` de la fiche, du registre ou du code), **la ligne lue citée** (sortie `grep -nF` ou extrait
   copié) qui fonde l'affirmation, et le remède proposé. Une affirmation de présence ou d'absence sans sortie `grep -nF`
   copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés** de ta lentille, un par un, avec, pour chaque axe exercé, ce que tu as
   lu ou cherché. Un « 0 finding » sans cette liste ne clôt rien et ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit (`issue create|edit|close|comment`, `pr …`) ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `make`, ni `latexmk`, ni
  `python3`, ni `node`, ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Rien dans `/home/gcorbaz/devel/kesh-0162` ni dans `/home/gcorbaz/devel/kesh` : lecture seule.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
