# Story 25.4-c4-b : L'arrondi à 5 centimes, visible et réglable

Status: done

**Issue : [#494]** (CR) — ⛔ la PR commune c4-a + c4-b porte `closes #494`, titre ET corps.

**Seconde des deux stories de la 25-4-c4.** La c4-a (done, même branche
`story/25-4-c4-arrondi-facture-5-centimes`) a posé le fond : l'arrondi est figé à la validation
(`invoices.rounding_amount`, `credit_notes.rounding_amount`), écrit en ligne finale de l'écriture de vente,
intégré à toutes les formes du TTC, et le réglage `company_invoice_settings.round_to_5_centimes` existe en base
(actif par défaut), sans API ni écran. **Depuis la c4-a, le PDF affiche un total arrondi sous un sous-total et
une TVA qui ne s'additionnent plus** : cette story rend l'écart visible et le réglage modifiable.

Arbitrages de Guy (2026-10-01, #494) : réglage par société actif par défaut ; ligne « Arrondi » sur la
facture (123.44 → +0.01 → 123.45 ; 234.52 → −0.02 → 234.50) ; avoirs idem ; ligne sans TVA.

## Story

En tant que comptable suisse,
je veux voir la ligne d'arrondi sur la facture et pouvoir activer ou désactiver l'arrondi à 5 centimes,
afin que le total imprimé s'explique ligne par ligne et que je choisisse la règle de ma société.

## Les faits, vérifiés dans le code

- **Le gabarit PDF** (`crates/kesh-qrbill/src/pdf.rs:640-704`, partagé par la facture, l'avoir et le rappel,
  `draw_invoice_section`) affiche « Sous-total HT », la TVA par taux et « Total TTC » si la facture porte de la
  TVA (`!inv.vat_lines.is_empty()`), sinon le total seul. La hauteur du récapitulatif est **réservée**
  (`recap_reserve`, `pdf.rs:589-593`, 4.5 par ligne) pour la garde `TooManyLines`.
- **Les données du PDF** (`crates/kesh-qrbill/src/types.rs:146-154`, `InvoicePdfData` : `subtotal_ht`,
  `vat_lines`, `total`) sont remplies par `invoice_pdf_service.rs` (facture, rappel) et `routes/credit_notes.rs`
  (avoir). Le `total` lit déjà l'arrondi figé depuis la c4-a.
- **La réponse facture** (`routes/invoices.rs:206`, `InvoiceResponse`, construite par `from_parts` puis
  `with_settlement`) porte `total_amount` (HT), `total_ttc` (arrondi figé compris), `vat_breakdown`, mais pas
  l'arrondi lui-même.
- **La fiche facture** (`frontend/src/routes/(app)/invoices/[id]/+page.svelte:988-1011`) affiche « Sous-total
  HT » → TVA par taux → « Total TTC » (valeurs serveur).
- **Le réglage** : `CompanyInvoiceSettings.round_to_5_centimes` (entité, `COLUMNS`, instantané d'audit) existe ;
  ni `CompanyInvoiceSettingsUpdate`, ni l'`UPDATE` du dépôt (`company_invoice_settings.rs:182`), ni
  `is_no_op_change` (`:116`), ni la route (`routes/company_invoice_settings.rs:45, 90, 274-301`), ni l'écran
  (`settings/invoicing/+page.svelte:349-360`, section *Différences d'arrondi*) ne le portent.

## Acceptance Criteria

**AC 1 — La ligne « Arrondi » du PDF.** Quand `rounding_amount != 0`, le récapitulatif affiche, entre la TVA
et le « Total TTC », une ligne **« Arrondi »** avec l'écart signé (`+0.01`, `−0.02`), au centime. Le bloc
« Sous-total HT » s'affiche alors **même sans TVA** : le total doit s'expliquer. `recap_reserve` compte cette
ligne (et le sous-total qu'elle fait apparaître) ; la garde `TooManyLines` reste juste. Facture, **avoir** et
**rappel** (même gabarit) en bénéficient ; un rappel montre l'arrondi de sa facture. Champ neuf
`InvoicePdfData.rounding` (`Decimal`, `0` = pas de ligne). Libellé `invoice-pdf-rounding` dans les 4 locales
(catalogue du PDF, patron de `invoice-pdf-total-ttc`).

**AC 2 — L'API de la facture.** `InvoiceResponse` porte `roundingAmount` :
- facture validée ou annulée : l'arrondi **figé** ;
- **brouillon** : un **aperçu**, calculé par `vat::invoice_rounding` d'après le réglage **courant** de la
  société, avec `roundingIsPreview: true`. Le `totalTtc` d'un brouillon reste celui des lignes ; l'aperçu
  n'altère aucun calcul ni aucune liste.

Construit par un builder du patron de `with_settlement` (par ex. `with_rounding_preview(round_to_5_centimes)`),
appliqué par **chaque handler qui renvoie un brouillon** : `get_invoice`, et aussi la **dévalidation**
(`unvalidate_invoice_handler`, `invoices.rs:847-866`), dont la fiche consomme la réponse **sans relire** la facture
(`+page.svelte:310-319`) — sans le builder, une facture tout juste redevenue brouillon montrerait `roundingAmount: 0`
jusqu'au rechargement (validation P1, HIGH) —, ainsi que la création et la modification d'un brouillon, par
cohérence. Les réponses d'une facture validée (règlement, e-mail, validation) portent l'arrondi figé, sans
builder. Type frontend (`invoices.types.ts`) aligné.

**AC 3 — La fiche facture.** Entre la TVA et le « Total TTC », une ligne **« Arrondi »** si `roundingAmount
!= 0` ; sur un brouillon, **« Arrondi (estimé) »** et un total estimé (`totalTtc + roundingAmount`), libellé
« Total TTC (estimé) ». Comme au PDF, le sous-total s'affiche dès qu'il y a un arrondi. `data-testid` sur la ligne
(garde `e2e-selecteurs-traduits`).

⛔ **Les libellés du récapitulatif passent tous en `i18nMsg`** — « Sous-total HT », « TVA {taux} % », « Total TTC »,
« Total », plus les trois neufs —, dans les 4 locales, `sitesTotal` recompté. Ils sont aujourd'hui en français codé
en dur (`+page.svelte:988-1011`) ; y glisser trois libellés traduits mélangerait deux conventions dans une même
table (validation P1). ⚠️ Les **en-têtes de colonnes** du tableau des lignes restent codés en dur : c'est eux que
vise la convention écrite à côté (`+page.svelte:959-964`, AC6-bis), et le commentaire est complété pour dire que le
récapitulatif, lui, est désormais traduit.

**AC 4 — Le réglage, API et écran.**
- `CompanyInvoiceSettingsUpdate.round_to_5_centimes: bool` — la valeur **résolue** que le dépôt écrit (le corps
  de la requête, lui, porte un `Option<bool>`, ci-dessous), l'`UPDATE` et `is_no_op_change`
  (`company_invoice_settings.rs:116`) le portent ; l'audit suit sans code neuf (l'instantané `before`/`after`
  relit `COLUMNS`, qui porte déjà le champ) ;
- le doc-comment de `default_rounding_account_id` (`entities/company_invoice_settings.rs:41`, « Facultatif : il
  n'est lu que quand un écart se présente ») est rectifié comme le manuel l'a été en c4-a : l'arrondi à 5
  centimes, actif par défaut, rend ce compte nécessaire à la validation de la plupart des factures. ⚠️ Les
  autres « facultatif » du dépôt (`company_invoice_settings.rs:251, 439, 567`, test `:1071`) parlent de la
  **désignation à la création** d'une société, où l'absence n'est pas une erreur : ils restent justes ;
- la route : `roundTo5Centimes` en `GET` et en `PUT` — **absent du corps, la valeur en place est préservée**
  (`Option<bool>`, patron du compte d'arrondi) ;
- l'écran : dans la section *Différences d'arrondi*, une case **« Arrondir le total des factures émises à 5
  centimes »** avec une aide qui dit l'effet (ligne « Arrondi », écart sur le compte ci-dessous, factures déjà
  émises inchangées). Libellés dans les 4 locales, `data-testid`.
- ⛔ **Changer le réglage ne touche aucune facture émise** (l'arrondi est figé) : l'aide le dit, un test le
  prouve.

**AC 5 — Textes.**
- `user-manual.tex`, là où il décrit la facture et son total : l'arrondi à 5 centimes, la ligne « Arrondi », le
  réglage, l'aperçu en brouillon.
- `admin-manual.tex` (*Compte de différences d'arrondi*, réécrit en c4-a) : nommer l'interrupteur.
- PDF régénérés, contrôlés aplatis.
- CHANGELOG `[0.12.1]` : l'entrée *Changed* de la c4-a, complétée (ligne visible, réglage), sans la mention
  « suivent ».

**AC 6 — Tests.** Chacun aurait échoué avant le patch :
- PDF : une facture arrondie porte la ligne « Arrondi » et un sous-total, même sans TVA ; facture non arrondie
  inchangée ; avoir et rappel aussi ; la garde `TooManyLines` compte la ligne (test du patron existant) ;
- API : `roundingAmount` figé (validée), aperçu (brouillon, `roundingIsPreview`), aperçu nul réglage désactivé ;
- réglage : `PUT` qui change `roundTo5Centimes`, `PUT` sans le champ qui le préserve, audit ; une facture émise
  garde son arrondi après désactivation ;
- Vitest : la fiche (ligne, « estimé » en brouillon, absente sans arrondi) et l'écran des réglages ;
- **E2E** : désactiver puis réactiver le réglage dans *Paramètres → Facturation*, et voir la ligne
  « Arrondi » sur la fiche d'une facture validée.

## Tasks / Subtasks

- [x] **T1 — le PDF** (AC 1) : `InvoicePdfData.rounding`, gabarit, `recap_reserve`, trois remplisseurs, libellé.
- [x] **T2 — l'API de la facture** (AC 2).
- [x] **T3 — la fiche** (AC 3).
- [x] **T4 — le réglage** (AC 4) : dépôt, route, écran.
- [x] **T5 — textes** (AC 5).
- [x] **T6 — tests** (AC 6).
- [x] **T7 — gates** : backend complet (base remise à zéro ; `kesh-db` touché), frontend complet, **E2E
  complet** (avant le push de la PR commune).

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **Recalculer l'arrondi d'une facture émise** pour l'afficher : on lit `rounding_amount`, figé.
- ⛔ **Calculer l'aperçu du brouillon dans le frontend** : une seconde définition de la règle. Le serveur la
  donne (`invoice_rounding`).
- ⛔ **Changer le `totalTtc` d'un brouillon** : l'aperçu est un champ à part, que seule la fiche lit.
- ⚠️ La liste des factures affiche toujours le HT dans sa colonne « Total » (défaut antérieur, relevé à
  l'inventaire) : hors périmètre.
- ⚠️ **L'écran d'un avoir** (`credit-notes/[id]/+page.svelte`) n'affiche que le total HT — ni TVA, ni TTC, ni
  arrondi —, et `CreditNoteResponse` n'expose rien de plus : défaut antérieur, hors périmètre (l'AC 3 vise la
  fiche facture). Le PDF de l'avoir, lui, porte la ligne « Arrondi » (revue de code P1, lentille C).

### Modules

`kesh-qrbill`, `kesh-api`, `kesh-db` (le réglage), `frontend`, `kesh-i18n` (+ `docs`) — cinq modules de code,
au seuil sans le dépasser **au compte par crate**. ⚠️ Au compte par module métier, le frontend en touche deux
(fiche facture, paramètres de facturation), soit six : signal faible, soumis à Guy (validation P1).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`).

### Debug Log References

- Aucun test du dépôt n'extrait le texte d'un PDF : le récapitulatif est devenu une **fonction pure**
  (`recap_lines`), patron du bloc du rappel ; le dessin et la réserve `TooManyLines` lisent la même liste, ce qui
  les garde d'accord par construction (la réserve d'avant est reproduite à l'identique sans arrondi).

### Completion Notes List

- **PDF (AC 1)** : `InvoicePdfData.rounding` ; `recap_lines` (sous-total, TVA par taux, « Arrondi » signé par
  `format_signed_ch`) affichée dès qu'il y a TVA **ou** arrondi ; clé `invoice-pdf-rounding` (FTL ×4 et
  `I18N_KEYS`/`DEFAULT_EN`, en fin) ; remplisseurs : facture et rappel (`invoice.rounding_amount`), avoir
  (`cn.rounding_amount`).
- **API (AC 2)** : `roundingAmount`, `roundingIsPreview` ; `with_rounding_preview` + `with_draft_rounding_preview`
  (réglages lus seulement pour un brouillon), appliqué à la lecture, la création, la modification et la
  **dévalidation**.
- **Fiche (AC 3)** : ligne « Arrondi » / « Arrondi (estimé) », total estimé (big.js) ; le récapitulatif entier
  passé en `i18nMsg` (sept sites, sept clés ×4) ; commentaire de convention complété.
- **Réglage (AC 4)** : `CompanyInvoiceSettingsUpdate.round_to_5_centimes`, `UPDATE`, `is_no_op_change` ; route
  `roundTo5Centimes` (`Option<bool>` préservé) ; case à cocher et aide ; aide du compte d'arrondi actualisée (elle
  ne parlait que du demi-centime) ; doc-comment de `default_rounding_account_id` rectifié.
- **Textes (AC 5)** : manuel utilisateur (paragraphe *Le total arrondi à 5 centimes* sous la validation), manuel
  admin (l'interrupteur nommé), PDF régénérés et contrôlés aplatis ; CHANGELOG complété (la mention « suivent »
  retirée).
- **Tests** (périmètre : ce commit contre `becfe6b2`) : **7** Rust neufs (2 `kesh-qrbill`, 4
  `invoice_echeancier_e2e.rs`, 1 `idor_multi_tenant_e2e.rs`), **4** Vitest, **1** spec Playwright neuve
  (`invoice-rounding-5-centimes.spec.ts`) ; `sitesTotal` 1757 → 1766 recompté (fiche 69 → 76, réglages 36 → 38).
- **Mutations**, toutes tuées : ligne d'arrondi retirée du récapitulatif ; aperçu absent de la dévalidation ;
  réglage ignoré par la route ; total estimé sans l'aperçu.
- **Gates** : backend complet sur base remise à zéro — **2566/2566** ; frontend complet — 0 erreur, lint PASS,
  **849/849**, build ; **E2E complet** sur `kesh_e2e` reconstruite, 09:18 UTC — **227 passés, 19 ignorés,
  10 échecs**, tous expliqués : les 7 KF-029, les 2 KF-045 (avant 12:00 UTC), et `products.spec.ts:109`,
  pollution d'état (vert rejoué seul, avec la spec neuve, elle aussi verte).

### File List

- `crates/kesh-qrbill/src/{pdf,types}.rs`, `crates/kesh-qrbill/tests/golden_test.rs`
- `crates/kesh-api/src/routes/{invoices,invoice_pdf_service,credit_notes,company_invoice_settings}.rs`
- `crates/kesh-api/tests/{invoice_echeancier_e2e,idor_multi_tenant_e2e}.rs`
- `crates/kesh-db/src/entities/company_invoice_settings.rs`, `crates/kesh-db/src/repositories/company_invoice_settings.rs`,
  `crates/kesh-db/tests/company_invoice_settings_repository.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `frontend/src/lib/features/invoices/invoices.types.ts`
- `frontend/src/routes/(app)/invoices/[id]/+page.svelte`, `…/invoice-settlements-page.test.ts`
- `frontend/src/routes/(app)/settings/invoicing/+page.svelte`, `…/settings-invoicing-page.test.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/tests/e2e/invoice-rounding-5-centimes.spec.ts` (neuf)
- `docs/manual/fr/{admin,user}-manual.tex` + `.pdf`, `CHANGELOG.md`, `sprint-status.yaml`

## Change Log

- **2026-10-01** — Créée : ligne « Arrondi » au PDF (facture, avoir, rappel) et à la fiche, aperçu en brouillon
  calculé par le serveur, réglage exposé (API qui préserve l'absent, case à cocher), manuels, CHANGELOG.
- **2026-10-01** — Validation P1 (Sonnet) : 1 HIGH, 2 MED, 3 LOW, tous retenus. **HIGH** : la dévalidation renvoie un
  brouillon que la fiche affiche sans relecture — l'aperçu s'applique à toute réponse qui rend un brouillon.
  **MED** : doc-comment du compte d'arrondi resté « facultatif » après la correction du manuel en c4-a — ajouté au
  T4 ; libellés du récapitulatif codés en dur — passés tous en `i18nMsg` plutôt que mélanger. **LOW** : l'audit
  suit sans code ; référence `:116` ; compte des modules signalé à Guy.
- **2026-10-01** — Validation P2 (Haiku) : 1 MED rendu, **réfuté** — il confondait le corps de la requête
  (`Option<bool>`, l'absent préservé) et la valeur résolue que le dépôt écrit (`bool`), comme pour le compte
  d'arrondi ; la phrase est clarifiée. Axe 0 recontrôlé par l'orchestrateur : les autres « facultatif » du dépôt
  portent sur la désignation à la création, légitimes. **Boucle close** : 1 HIGH/2 MED/3 LOW → 0 réel ;
  Sonnet → Haiku.
- **2026-10-01** — Implémentée (T1–T7) : ligne « Arrondi » au PDF (récapitulatif en fonction pure), API à aperçu
  sur toute réponse brouillon, fiche traduite, réglage exposé et case à cocher, manuels, CHANGELOG. 7 tests Rust,
  4 Vitest, 1 spec Playwright neufs ; 4 mutations tuées. Gates : backend 2566/2566, frontend 849/849, E2E 227/19/10
  expliqués.
- **2026-10-01** — Revue de code P1 (Sonnet ×3, prompt `25-4-c4-b-review-prompt-p1.md`) : A 1 MED/2 LOW, B 1 MED,
  C 1 MED/2 LOW — tous retenus. **B** : l'aperçu lisait le réglage par `get_or_create_default`, dont l'`INSERT
  IGNORE` ouvrait une transaction d'écriture à chaque `GET` de brouillon, clés d'API en lecture seule comprises →
  lecture pure `company_invoice_settings::round_to_5_centimes` (défaut actif si la ligne manque), test « une lecture
  n'écrit rien ». **A** : un écart infra-centime (les lignes portent quatre décimales) imprimait « +0.00 » → ligne et
  signe décidés au centime, au PDF comme à la fiche, tests (0.0004 sans ligne, −0.0050 → « −0.01 ») ; doc-comment de
  `format_ch` remis à sa place, commentaire citant un `show_recap` inexistant corrigé. **C** : le manuel utilisateur
  plaçait la ligne « Arrondi » sur la QR-facture — le bulletin de versement ne porte que le montant arrondi →
  corrigé, PDF régénéré ; l'écran d'un avoir (HT seul) documenté comme défaut antérieur hors périmètre ; tests du
  PDF de l'avoir et du rappel, que l'AC 6 promettait sans les avoir. Gates : backend **2569/2569**, frontend
  **850/850** ; E2E rejouée au push.
- **2026-10-01** — Revue de code P2 ciblée (Haiku, `25-4-c4-b-review-prompt-p2.md`, sur `44925eea`) : 1 LOW rendu,
  **réfuté par exécution** — la lentille affirmait que big.js `roundHalfUp` arrondit −0.005 à 0.00 ; `node -e` rend
  −0.01 (loin de zéro, comme `MidpointAwayFromZero`), et −0.0249 → −0.02. Les quatre axes déclarés, aucune
  remédiation. **Boucle close** : P1 3 MED/4 LOW → P2 0 réel ; Sonnet ×3 → Haiku. Gate backend complet au dernier
  commit de code : **2569/2569** ; frontend **850/850** ; E2E rejouée avant le push de la PR commune.

[#494]: https://github.com/guycorbaz/kesh/issues/494
