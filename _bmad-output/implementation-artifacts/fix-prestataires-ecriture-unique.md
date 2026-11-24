# Correctif : une transaction de prestataire n'est écrite qu'une fois

Status: review

*Fiche rédigée le 2026-11-24, branche `fix/prestataires-ecriture-unique`, partie de `origin/main` à `f4e64b179`.
Correctif du moteur commun des prestataires (Epic 38, déjà publié), non urgent, **sans migration**. Relevé par la
relecture de la PR de l'Epic 40 (M2, versement Stripe), puis étendu par le mainteneur à PayPal. Issues P1 : #1538
(PayPal), #1539 (Stripe, avec la PR de l'Epic 40).*

## Intention

Annuler le rapprochement du **côté prestataire** d'un virement dont l'autre côté est rattaché ne retire que le lien
(`reconciliation_cancel::entry_cancel_outlook`, rang 0, `unlink_only: true`, par `transfer_other_side`) : l'écriture
reste, rapprochée de l'autre côté. La transaction redevient `pending` ; la revue commune la propose de nouveau
(`provider_review::build_review`, retrait ; règle) et l'acceptation l'écrit une seconde fois
(`write_provider_entry`), le rejugement ne contrôlant que l'état « à rapprocher » (`rejudge`). Le compte de l'autre
côté (la banque, la carte) est alors compté deux fois.

**Prémisse vérifiée au code** (`f4e64b179`) : `rejudge` (`routes/provider_accept.rs`) ne lit que l'état, le genre, le
compte ; `accept_withdrawal` et `accept_rule` écrivent sans lire d'écriture existante ; `build_review` propose
`withdrawal` pour tout retrait CHF pending et `rule` pour toute transaction `Other` qu'une règle reconnaît.

## Décision

1. **La règle, dans le moteur commun** : une transaction pending est **déjà comptabilisée** quand une écriture
   non contre-passée (ni contre-passation), hors règlement, porte sur le compte du plan de son solde un mouvement égal
   à son montant **et** est déjà rapprochée d'une transaction d'un **autre** compte bancaire, de sens contraire
   (l'autre côté d'un virement). C'est, mot pour mot, `reconciliation_link::find_matched_elsewhere` — la règle du
   signal « déjà comptabilisée » de la réconciliation bancaire (34-4) et du retrait du seul lien à l'annulation
   (34-11b) : **une seule définition**, aucun texte de libellé.
2. **Revue** : `load_review_in_tx` lit, par compte bancaire du prestataire, `matched_elsewhere_for_page` (nouvelle
   lecture par page, du même texte SQL que `count_already_booked_for_page`, refactorisé en `page_entry_rows`) ;
   `build_review`, **avant tout autre motif**, met « à examiner » sous `already_posted` (`details: { journalEntryId }`,
   la plus proche par date, puis la plus ancienne — l'ordre de `find_matched_elsewhere`) une transaction d'un genre
   qui **écrit** — frais, conversion, retrait, autre (règle) ; `fee_account_missing` et `no_rule` ne la masquent pas.
   Une moitié de conversion déjà comptabilisée ne s'apparie plus. **Non touchés** : un genre admis au rattachement
   (`admits_link` : l'encaissement, et ce que le prestataire déclare par `ProviderSpec::linkable`) et les genres
   propres (`Specific`), que le prestataire décide lui-même.
3. **Acceptation** : `rejudge_writing` = `rejudge` + `find_matched_elsewhere` sous les verrous du lot, appelé par
   `accept_fee`, `accept_rule`, `accept_withdrawal` et `accept_conversion` (chaque moitié) ; déjà comptabilisée →
   `FailedProposal` `PROVIDER_TRANSACTION_ALREADY_POSTED` (`details: { journalEntryId }`), `200`, `failed[]`, rien
   n'est écrit (*Pattern batch*).
4. **Pourquoi pas le libellé** : les libellés PayPal portent l'identifiant (`… · TXN <id>`), mais l'écriture des frais
   d'une transaction porte le **même** identifiant sans `:FEE` ; et un libellé ne relie pas l'écriture à son
   mouvement. Le lien structuré (l'écriture, ses lignes et ses transactions) existe et couvre exactement le seul
   chemin qui laisse une écriture vivante à une transaction pending : le retrait du seul lien, qui suppose un autre
   côté rattaché.
5. **Angle mort assumé** : une écriture passée à la main pour la transaction et **pas encore rattachée** n'est pas
   vue (elle n'a pas d'autre côté). L'étendre aux écritures non rattachées ferait passer « à examiner » toute
   transaction dont un mouvement de même montant traîne au compte du plan (saisies manuelles d'avant l'import) : faux
   positifs, jugé plus coûteux. Écrit au manuel.

## Critères

- AC-1 (retrait) : retrait accepté, réception de la banque rattachée, côté PayPal annulé → revue : non proposé,
  `already_posted` avec l'écriture ; l'ancienne proposition renvoyée → `PROVIDER_TRANSACTION_ALREADY_POSTED`, aucune
  écriture ni trace ; le rattachement à l'écriture passe.
- AC-2 (règle vers une carte) : même scénario, règle dont la contrepartie est le compte du plan d'une carte.
- AC-3 (sonde positive) : un retrait jamais écrit reste proposé et s'accepte, même quand une écriture du même montant,
  rapprochée de la banque mais sans ligne au compte PayPal, existe.
- AC-4 : conversion à une écriture, une moitié détachée → `already_posted`.
- AC-5 : quatre locales, manuel utilisateur, `docs/api-external.md`.

## Reprise sur `pr/40-stripe` (versement Stripe)

Après la fusion de cette branche dans `main` et sa reprise dans `pr/40-stripe` : dans `accept_payout`, remplacer
`rejudge(` par `rejudge_writing(` ; dans la revue Stripe, passer `loaded.already_posted` à `StripeReviewInput` et,
dans `decide_payout`, avant tout autre motif, pousser `ReviewFinding::already_posted(t, id)` quand la transaction y
figure. Le correctif propre à Stripe (`b99591697`) se retire alors (la règle textuelle `po_…` n'a plus d'objet).

## Change Log

- 2026-11-24 — correctif (`62d8783bb`), sonde positive (`0617f7ef3`), remédiation de la revue P1 (`4d9093c34`).
  Revues : P1 complète (Opus) 0/0/3/7 → remédiée (M1 motif en tête, M2 tests d'égalité et de non-régression, M3
  textes ; L1, L4, L5, L6, L7) ; P2 ciblée (Sonnet) sur `4d9093c34` : 0/0/0/4, boucle close. LOW restants, à tracer :
  `writes` exempte un genre `admits_link` qui n'est pas rattaché par le constructeur générique (latent) ; genre propre
  `ruleable` de Wise à rejuger à l'intégration de l'Epic 39 ; nom du test « même identifiant » ; liste des motifs
  remplacés dans `api-external.md` ; coût de `find_matched_elsewhere` par proposition à l'acceptation (P1 L2) ; refus
  d'une conversion déjà comptabilisée inatteignable (sa partenaire est rapprochée, d'où `PROVIDER_PROPOSAL_STALE`
  d'abord, P1 L3). Mutations : cinq, toutes tuées par une assertion. Gates au commit `4d9093c34` : gate-ci 6451/0/19,
  E2E 505 passés et les sept de la KF-029, Vitest 5008. Journal : `kesh-gate-logs/fix-prest/`.

## Notes de version

- **Fixed** — Prestataires (PayPal) : une transaction déjà comptabilisée de l'autre côté — typiquement un retrait, ou
  un paiement imputé par une règle à une autre banque ou à une carte, dont seul le côté PayPal a été annulé — n'est
  plus proposée à la revue : elle est « à examiner » (*Déjà comptabilisée*), et son acceptation est refusée sans rien
  écrire (`PROVIDER_TRANSACTION_ALREADY_POSTED`). Le compte de l'autre côté n'est plus compté deux fois.
