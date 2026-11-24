# Correctif : un versement Stripe déjà écrit n'est plus proposé ni écrit une seconde fois

Status: review

*Fiche rédigée le 2026-11-24, branche `pr/40-stripe`, partie de `b29ce763e`. Correctif de l'Epic 40 avant la fusion
de sa PR, **sans migration**. Relevé par la relecture de la PR par l'agent du mainteneur (M2, jugé de classe P1).*

## Intention

Annuler le rapprochement **côté Stripe** d'un versement dont la réception bancaire est déjà rattachée ne retire que le
lien (`reconciliation_cancel.rs`, `entry_cancel_outlook`, rang 0 : `unlink_only: true`) : l'écriture du versement
reste, pointée par la réception. Le versement redevenait `pending`, la revue le proposait de nouveau (`decide_payout`)
et l'acceptation (`accept_payout`) ne cherchait aucune écriture existante du même `po_…` : l'accepter passait une
seconde écriture, la banque débitée deux fois. Seul garde-fou : une phrase du manuel.

**Prémisse vérifiée au code** (`b29ce763e`) : `accept_payout` (`routes/stripe.rs`) enchaînait `rejudge`, la
destination, `judge_payout`, `write_provider_entry` sans aucune lecture d'écriture existante ; `rejudge` n'exige que
`pending` et le genre.

## Décision

1. **Règle** : un versement dont l'identifiant `po_…` figure, en **mot entier** et casse comprise, au libellé d'une
   écriture **vivante** (ni contre-passée, ni contre-passation), rapprochée ou non, n'est **pas proposé** : il va
   « à examiner » sous `payout_already_posted` (`details: { payoutId, journalEntryId }`), avant tout autre motif.
2. **Acceptation** : la même règle, rejugée sous les verrous du lot, juste après `rejudge` → `FailedProposal`
   `STRIPE_PAYOUT_ALREADY_POSTED` (`details: { payoutId, journalEntryId }`), `200` et `failed[]` (*Pattern batch*),
   rien d'écrit.
3. **Le lien : textuel, faute de lien structuré.** `journal_entries` n'a aucune colonne de source ; le seul lien entre
   l'écriture et le versement était `bank_transactions.matched_entry_id` du versement, précisément celui que
   l'annulation retire. Reste le libellé, que l'acceptation écrit depuis celui de la transaction (« Versement Stripe
   po_… », ou « Stripe <compte> · po_… »). Recherche `LOCATE` en collation binaire (jamais `LIKE`), mot entier jugé en
   Rust (`posted_payouts`, seule définition, appelée par la revue et l'acceptation via `already_posted`).
4. **Exclusions** : les écritures des **frais** portent aussi le `po_…` de leur versement (« … - po_… », 40-7b) ;
   elles sont écartées parce qu'une transaction d'un compte Stripe les pointe (l'écriture d'un versement détaché n'est
   plus pointée que par sa réception, banque ou Wise). Une écriture contre-passée ne compte pas : l'annulation d'un
   versement **sans** réception contre-passe son écriture, et le versement doit redevenir proposable.
5. **Limite assumée** : une écriture saisie à la main qui cite le versement compte aussi ; le versement passe alors à
   examiner (sens sûr). La lecture des écritures n'est pas verrouillante : une écriture saisie au journal en même temps
   qu'une acceptation n'est pas vue (le verrou du compte exclut les acceptations et rattachements concurrents).

## Critères

- AC-1 : versement accepté, réception rattachée, côté Stripe annulé → revue : non proposé, à examiner
  `payout_already_posted` avec l'écriture ; l'ancienne proposition, acceptée de force → `STRIPE_PAYOUT_ALREADY_POSTED`,
  aucune écriture ni trace ; le rattachement de la transaction à l'écriture passe.
- AC-2 (sonde positive) : un versement jamais écrit reste proposé, même après l'acceptation de ses frais (dont les
  écritures portent son `po_…`), et s'accepte.
- AC-3 : un versement dont l'écriture a été contre-passée redevient proposable (test existant
  `annulation_d_un_versement_avant_reception`).
- AC-4 : textes aux quatre locales, manuel utilisateur et `docs/api-external.md`.

## Change Log

- 2026-11-24 — correctif, tests, textes ; trace des revues et des gates au journal `kesh-gate-logs/s40-m2/`.

## Notes de version

- **Fixed** — Stripe : un versement dont l'écriture existe déjà (par exemple après l'annulation de son seul côté
  Stripe, sa réception restant rattachée) n'est plus proposé à la revue : il est « à examiner » (*Versement déjà
  écrit*), et son acceptation est refusée sans rien écrire (`STRIPE_PAYOUT_ALREADY_POSTED`). La banque n'est plus
  débitée deux fois.
