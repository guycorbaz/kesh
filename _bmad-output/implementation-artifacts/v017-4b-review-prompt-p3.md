# Revue de code P3 — Story v017-4b — passe CIBLÉE sur la remédiation de la P2 — #849

*Prompt versionné (`CLAUDE.md` § *La passe ciblée* : une seule lentille, en contexte frais, braquée sur le seul
diff de la remédiation précédente). Lentille **R** (Regression Hunter), modèle **Opus**. Langue : français soigné.*

## Ce qui précède

- Revue P2 (Sonnet, lentilles B, E, A) : `/home/gcorbaz/devel/kesh-gate-logs/v0174b-review-p2-{B,E,A}.md` —
  2 MEDIUM convergents : **B2-1 = A2-1** (l'avertissement du lot confirmé affirmait « payée par le lot …, puis son
  règlement a été annulé », que `payment_batches.rs:903-904` interdit) et **B2-2 = A2-2 = E2-1** (les refus du cycle,
  rendus hors des blocs de statut, survivaient à *Payer*, *Annuler le règlement*, *Annuler la facture*) ; des LOW.
- Remédiation : commits `fa71e7bc` et `48b39fbd`, puis le compte rendu. La table « finding P2 → sort » est au Dev
  Agent Record de `v017-4b-facture-fournisseur-brouillon-ecran.md` (§ *Remédiation de la revue de code P2*) ;
  annotations datées de C-v017-4-63 et C-v017-4-64 au registre `v017-4-choix-autonomes.md`.

## Le périmètre — le seul diff de la remédiation

`git diff f8b119e5..HEAD` **hors** `_bmad-output/`. Il porte : `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte`
(`confirmedBatchFact`, `confirmedBatchAction`, condition du bloc de la facture validée, `clearCycleErrors()` dans
`pay`, `cancelSettlement`, `cancel`, date suisse dans la confirmation d'annulation du règlement), les quatre
catalogues (`supplier-invoices-unvalidate-batch-warning`, `-undated`, `-action` ; it-CH
`error-supplier-invoice-unvalidate-blocked-*` à la forme « tu »), `SupplierInvoiceForm.svelte` (étiquette du taux
expiré), le manuel utilisateur `.tex` **et** son PDF, le CHANGELOG, les bancs Vitest, la spec E2E du cycle, les
gardes i18n (`sitesTotal` 2794 → 2796, `CLES_RELEVEES` 387 → 389). Lisez au-delà du diff ce qu'il faut pour juger
(le reste de la fiche, `payment_batches.rs`, `supplier_invoices.rs`), mais ne rapportez que ce que **la remédiation**
introduit ou laisse faux.

## Les questions

1. **Le texte du lot.** Dit-il désormais un fait que le serveur établit, et seulement lui — aux quatre locales, au
   manuel (tex et PDF aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`), au CHANGELOG,
   dans les bancs et l'E2E ? La consigne (« la facture a peut-être déjà été payée : vérifiez le relevé ») est-elle
   vraie dans tous les cas où elle s'affiche (facture revalidée après correction, `confirmedAt` nul) ? Le fait seul,
   pour la Consultation, se lit-il comme une phrase complète ? Les deux phrases se joignent-elles proprement (espace,
   ponctuation) à l'écran et dans le dialogue ? Le de-CH dit-il « Zahlungsauftrag » comme sa clé voisine ? Les
   traductions disent-elles la même chose que le français ?
2. **L'effacement des refus.** `clearCycleErrors()` est-il appelé au bon moment dans `pay` (avant la validation des
   champs), `cancelSettlement` et `cancel` (après la confirmation) ? Un refus utile peut-il désormais disparaître trop
   tôt (geste abandonné à la confirmation, `payError` d'un champ manquant) ? Un geste qui change le statut reste-t-il
   sans effacement ?
3. **Le bloc de la facture validée.** `{:else if invoice.status === 'open' && (canManage || invoice.lastConfirmedBatch)}` :
   un rôle ou un état perd-il quelque chose qu'il avait (le bouton *Dévalider*, un refus) ?
4. **Les bancs et les mutations.** Chaque correctif a-t-il un banc qui rougirait sans lui ? Vérifiez au journal :
   `/home/gcorbaz/devel/kesh-gate-logs/v0174b/mut-r2/mutations.log` (S1 à S9, S1-ftl) et `mutations-2.log` (S1-ftl,
   rejouée après le banc des catalogues). Le banc des catalogues (regex d'exclusion) peut-il être vert par construction
   ou rougir à tort ?
5. **Les décomptes et le Dev Agent Record** contre les journaux `p2-*.log` de `/home/gcorbaz/devel/kesh-gate-logs/v0174b/`
   (nextest, Vitest, check, build, E2E complet et ses échecs nominatifs, `make user`).

## Le rapport

`/home/gcorbaz/devel/kesh-gate-logs/v0174b-review-p3-R.md`. Chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW),
lieu (`fichier:ligne`), **ligne lue citée**, défaut, remède ; dire s'il vient de la remédiation P2. Toute affirmation
de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée. **Axes exercés et non exercés**, listés : un
« 0 finding » sans cette liste ne compte pas. Décompte final recompté depuis la liste.

## Les interdits — lecture seule stricte

- Aucune commande qui écrit, compile ou exécute : **ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni `python3`, ni
  `docker`, ni `curl`, ni `claude`**, ni aucun script du dépôt ou des journaux (`scripts/*`, dont
  `scripts/prepare-release.sh` ; `kesh-gate-logs/v0174b/*.sh`, `mut*/campagne*.sh`), ni `sed -i`, ni `rm`, `pkill`,
  `killall`.
- **Aucune redirection** (`>`, `>>`, `tee`), aucun fichier temporaire : un fichier créé par erreur se déclare au
  rapport, il ne se supprime pas. Le seul fichier écrit est le rapport, par l'outil d'écriture. `pdftotext` n'écrit que
  vers la sortie standard (`-`).
- Aucun `git checkout`, `switch`, `reset`, `stash`, `commit`, `push` ; `git diff`, `git show`, `git log` seulement.
- Aucune donnée réelle ; aucune base du NAS.
