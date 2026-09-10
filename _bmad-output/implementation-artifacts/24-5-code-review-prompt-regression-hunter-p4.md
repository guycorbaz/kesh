# Prompt de la passe 4 de revue de code — Story 24-5

**Versionné** conformément au § « La passe ciblée » du `CLAUDE.md`.

- **Modèle** : Haiku 4.5, contexte frais (P1 : Sonnet 4.6 ×2 + Haiku 4.5 · P2 : Opus 5 · P3 : Sonnet 4.6)
- **Cible** : le seul commit de remédiation `7dd7396e`
- **P3** : 1 CRITICAL, 0 HIGH, 3 MED, 2 LOW — dont **6 sur 6** nés de la remédiation de la P2

---

Tu es une lentille de revue de code, en **contexte frais**, sur le dépôt Kesh
(`/home/gcorbaz/devel/kesh` — comptabilité suisse, Rust + Svelte). Tu réponds en français.

CIBLE : le commit **`7dd7396e`**, remédiation de la passe 3 de revue de code de la Story 24-5
(#375).

⛔ **LIS-LE COMME UN DIFF UNIQUE**, jamais comme une séquence :

```sh
git show 7dd7396e --stat
git show 7dd7396e
```

*(Cette consigne n'est pas cosmétique : le § « Haiku-specific guardrails » du `CLAUDE.md` documente
que l'indexation des numéros de ligne d'un diff multi-commit produit des hallucinations
« patch non appliqué » sur ce modèle. Un diff unique supprime la cause.)*

⛔ **RÈGLE ABSOLUE — GREP GROUND-TRUTH.** Avant d'affirmer qu'un code **manque**, ou qu'un
anti-pattern **subsiste**, tu DOIS le vérifier dans le fichier courant avec `grep -nF` (fixed-string
obligatoire — le code Rust/TS est plein de métacaractères) et **citer la commande et sa sortie**.
Un finding non vérifié au sol sera écarté sans discussion. *Une hypothèse éliminée par raisonnement
n'est pas une hypothèse testée — et l'inverse est vrai aussi.*

⛔ **HYPOTHÈSE DE TRAVAIL, MESURÉE SUR CETTE STORY** : *le défaut que tu cherches vient d'être écrit
par la remédiation que tu relis.* En revue de spec, 11/13 puis 3/4 ; en revue de code, 5/6 à la
passe 2 et **6/6 à la passe 3**. Aucune décision d'origine n'a jamais été prise en défaut.

⛔ **ET L'ARTEFACT LE PLUS SUSPECT DU LOT A DÉJÀ ÉTÉ PRIS EN DÉFAUT QUATRE FOIS** : le rappel
d'exemptions périssables de `scripts/prepare-release.sh` et son test. Il vient d'être **entièrement
refondu** (`ExemptionBasis`, un exemple Rust, plus aucun grep). Cherche le cinquième tour.

## Ce que la remédiation a produit

1. **Deux gardes serveur** : `SELECT active` → `SELECT active, postable` dans
   `crates/kesh-db/src/repositories/invoice_settlements_write.rs` et
   `crates/kesh-db/src/repositories/supplier_invoices.rs` (branches
   `SettlementChoice::InternalAccount`).
2. **Un filtre d'écran** : `frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte`,
   `a.active` → `a.active && a.postable`.
3. **Deux tests négatifs neufs** (`invoice_settlement.rs`, `supplier_invoices_repository.rs`).
4. **Le registre d'exemptions refondu** : enum `ExemptionBasis { Durable, PerishableSince(i64) }`,
   troisième champ de `EXEMPT_MIGRATIONS`, deux tests (`every_exemption_declares_a_coherent_basis`,
   `the_release_script_sees_every_perishable_exemption`) remplaçant celui par marqueur.
5. **Un exemple neuf** : `crates/kesh-db/examples/perishable_exemptions.rs`.
6. **Le bloc de rappel de `scripts/prepare-release.sh`, entièrement réécrit** — il ne grepe plus
   rien et peut désormais **refuser la release** (`exit 1`).
7. **Le manuel utilisateur**, deux paragraphes (`:328` et `:338`), **PDF régénéré**.
8. **`CLAUDE.md`** : reformulation du point 6 de la liste de contrôle pré-release.
9. **L'issue #428** (CR pour l'amendement de l'AC 13), le journal de passe 3 et `sprint-status.yaml`.

## Tes axes d'attaque, par rendement attendu

- **LES DEUX GARDES NEUVES CASSENT-ELLES UN CAS LÉGITIME ?** C'est le risque le plus cher d'une
  remédiation qui restreint. Un compte de contrepartie **légitime** peut-il être non-`postable` ?
  Regarde ce que sont réellement les comptes utilisés là — `journal_account_id` d'un compte
  bancaire, comptes de compensation, comptes de liaison — et si l'un d'eux peut porter
  `postable = FALSE` dans un plan livré ou après un `PUT`. **Vérifie dans les JSON des trois plans
  comptables**, pas seulement dans le code. Une garde trop large refuserait un règlement que rien
  ne devrait empêcher.
- **RESTE-T-IL UN CINQUIÈME CHEMIN D'ÉCRITURE ?** La passe 3 en a trouvé un quatrième que quatre
  passes de spec et deux de revue avaient manqué. Énumère **exhaustivement** les appels à
  `journal_entries::create_in_tx` du dépôt et, pour chacun, dis s'il vérifie `postable`, si son
  écran filtre, et si le manuel le décrit correctement. Regarde en particulier ce que personne n'a
  cité : les avoirs (`credit_notes.rs`), les factures fournisseurs importées
  (`imported_supplier_invoices*`), les paiements, l'import CAMT.053, les écritures d'ouverture ou
  de soldes de départ.
- **LE MANUEL DIT-IL VRAI, AU QUATRIÈME TOUR ?** Trois passes l'ont pris en défaut dans trois sens
  différents. Lis les deux paragraphes édités **et leur voisinage**, vérifie **chaque** affirmation
  contre le code, et contrôle que le **PDF** correspond au `.tex` (⚠️ `pdftotext` coupe les lignes :
  aplatis le texte avant de grep, sinon tu produiras un faux négatif).
- **LE NOUVEAU BLOC DU SCRIPT PEUT-IL REDEVENIR MUET ?** C'est la question qui a coûté quatre tours.
  **Exécute ses morceaux.** Que se passe-t-il si `cargo run --example` **échoue** (crate qui ne
  compile pas, `cargo` absent, réseau) ? Le `|| true` avale-t-il l'échec, et le rappel
  disparaît-il alors en silence ? Le `while read ... <<< "$PERISSABLES"` se comporte-t-il bien sous
  `set -euo pipefail` quand la variable est vide ? Le `grep -c ... || true` dans une substitution ?
  Un dépôt sans tag ? Un `git ls-tree` sur un tag annoté ? `bash -n` **puis** exécution réelle.
- **LES DIX `Durable` SONT-ILS JUSTES ?** Le test refuse un `Durable` dont la justification contient
  « aucune version publiée ». Lis les **onze** justifications et vérifie qu'aucune n'argumente en
  réalité sur un fait daté sous une autre formulation — le test ne cherche qu'une seule tournure.
- **LES DEUX TESTS NÉGATIFS SONT-ILS HONNÊTES ?** Éprouve-les par mutation (tu peux exécuter
  `cargo nextest run -E 'test(...)'`). Assertent-ils la bonne erreur ? Vérifient-ils qu'aucune
  écriture ne subsiste ?
- **LES DÉCOMPTES** : 2299 tests (4 skipped), 740 frontend, 11 entrées au registre, 66 migrations,
  13 AC + 3 invariants + 8 tâches, 62 pages de PDF, « 6 findings sur 6 ». **Recompte depuis la
  source**, et vérifie que chaque total est cohérent avec sa ventilation.
- **L'issue #428** : ses affirmations sont-elles exactes (`gh issue view 428`) ?

## Règles de méthode

- Tu **peux** exécuter `cargo fmt --check`, `cargo check`, `cargo clippy`, `cargo nextest run -E`,
  `bash -n`, `pdftotext`, `gh issue view` — **sous `scripts/mem-guard.sh`** pour tout travail Rust.
- Tu **ne peux pas** lancer la suite complète ni Playwright : déclare-le plutôt que de supposer.
- **N'écris aucun fichier du dépôt, ne commite rien, ne modifie aucune issue.** Si tu éprouves un
  test par mutation, restaure l'état exact et vérifie-le par `git status --porcelain`.
- Lis `/home/gcorbaz/devel/kesh/CLAUDE.md` — « Migration breaking policy », « Review Iteration
  Rule », « Recompter ses propres comptes rendus », « Propagation post-patch »,
  « Haiku-specific guardrails ».

## Format

Sans préambule. Par finding : identifiant (P4-n), **sévérité**, **site**, **démonstration**
(commande + sortie), **conséquence**, **remède**. Puis : (a) tableau des sévérités ; (b) la part des
findings nés de la remédiation ; (c) ce que tu as vérifié et trouvé exact ; (d) tes limites.

⚠️ **Si tu ne trouves rien au-dessus de LOW, dis-le nettement** — c'est le résultat attendu d'une
boucle qui converge. **Ne fabrique pas de la sévérité pour justifier la passe**, et n'invente aucun
finding « patch non appliqué » sans l'avoir grepé.
