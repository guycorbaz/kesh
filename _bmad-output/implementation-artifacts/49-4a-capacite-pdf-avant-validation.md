# Story 49-4a : capacité du PDF de facture contrôlée avant la validation (#1364)

Status: in-progress

*Fiche issue du découpage de la `49-4-designation-longue-pdf` (décision de Guy du 2026-11-20, sur la recommandation de
l'orchestrateur du 2026-11-19 et le finding V1-6 de la validation P1 de la 49-4 :
`/home/gcorbaz/devel/kesh-gate-logs/e494/validation-49-4-p1.md`). Rédigée le 2026-11-20 (Opus 5.5) à partir de la fiche de la 49-4
(ses D4, D5, D7 et D10, ses AC5, AC7, AC9 à AC13, ses tests T6, T-PROP et T10), **restreinte au dessin actuel des lignes** ; les
findings de la validation P1 qui touchent ce périmètre sont remédiés ici, prémisse relue au code (§ *Findings de la validation P1*).
Release **0.20.0** (coupure jeudi 2026-11-26 au soir). **Aucune migration.** Code lu à la tête de la branche
`story/49-4-designation-longue-pdf` (`a1877ef9f`, identique à `3379a971a` pour `crates/`). Statut `backlog` : fiche écrite, **sa
propre passe de validation est à venir** (elle ne repasse pas toute la validation P1 de la 49-4). La sœur, `49-4b`, porte #1363 et
#1400.*

**Issue** : `refs #1364` ; **aucun mot-clé de fermeture** dans un commit (il ne s'écrit que dans le corps de la PR de la release).
#1110 est recoupée, non traitée (D3).

## Story

En tant que Guy, qui émet de vraies factures et les remet sur papier, je veux qu'une facture dont les lignes ne tiendraient pas
sur la page de son PDF soit **refusée à la validation**, avec un message qui dit quoi faire, quand le brouillon se corrige encore,
au lieu d'être validée — numéro attribué, écriture passée — puis refusée au premier téléchargement.

**Sans changer le PDF d'aucune pièce** : cette story ne touche pas le dessin. Une désignation d'une ligne chiffrée reste réduite à
44 caractères suivis de « … » (c'est #1363, à la 49-4b) ; un caractère que la police n'a pas disparaît encore (c'est #1400, à la
49-4b). Toute facture, tout avoir, tout rappel se rend **octet pour octet comme avant** la story.

**Condition de sortie de la 49-1c** (fichier d'épique, `:159` dans `kesh-e49`, règle réécrite par la décision de Guy du 2026-11-20 : « la 49-1c ne
sort pas sans la 49-4a ») : sans elle, un commentaire long rendrait une facture validée impossible à imprimer.

## Ce qui se lit au code (`a1877ef9f`)

- `crates/kesh-qrbill/src/pdf.rs:415` `fn draw_invoice_section(` : le seul dessinateur des lignes (facture, rappel, avoir).
- `pdf.rs:548` `    let mut ty = PAGE_H - 130.0;`, puis `ty -= 5.0;` (en-têtes de colonnes) et `pdf.rs:618` `    ty -= 3.0;` : le tableau
  commence toujours à 159 mm, quel que soit l'en-tête (un en-tête trop haut rend `HeaderOverflow`, `pdf.rs:572`). La capacité ne
  dépend donc **que** des lignes, du récapitulatif, du bloc de rappel et du plancher : elle se calcule sans contact, ni compte
  bancaire, ni société.
- `pdf.rs:631` `    let recap = recap_lines(inv, i18n);` ; `recap_lines` (`pdf.rs:1894`) ne lit que `vat_lines`, `subtotal_ht`, `rounding`
  (au centime, `pdf.rs:1898-1903`) et la devise ; la réserve vaut `4.5 × n + 1` mm si le récapitulatif a `n > 0` lignes, plus
  `reminder_block_height(inv.reminder.as_ref())` (`pdf.rs:1998`).
- `pdf.rs:644` `    let plancher_des_lignes = content_floor + 15.0 + recap_reserve;` ; `content_floor` = `SEP_Y` = `PAYMENT_H` = 105 mm pour une
  facture (`pdf.rs:40-42`, `:115`), `CONTENT_FLOOR_NO_QR` pour un avoir (`pdf.rs:175`).
- La garde, **avant chaque ligne dessinée** : `pdf.rs:651` `                for texte in lignes_de_commentaire(&line.description) {` puis
  `pdf.rs:653` `return Err(QrBillError::TooManyLines(inv.lines.len()));` pour chaque ligne de texte d'un commentaire ou d'un titre
  (49-1b, D6) ; `pdf.rs:665` (même refus) pour une ligne chiffrée, qui prend **exactement une** ligne imprimée (sa désignation est
  réduite par `truncate_display(&line.description, 45)`, `pdf.rs:668`). Pas de 5 mm (`ty -= 5.0`).
- `lignes_de_commentaire` (`pdf.rs:1182`) rend **zéro** ligne pour une description sans rien d'imprimable, et retire les lignes vides
  de tête et de fin (doc-comment `pdf.rs:1176-1181`) : un tel commentaire ne consomme aucun pas.
- Capacité d'une facture qui en découle (arithmétique de la validation P1, § *Ce qui tient*, à refaire à l'étape 0) : **8** lignes
  imprimées sans récapitulatif ; **6** avec un récapitulatif de deux lignes (sous-total et un taux, ou sous-total et arrondi) ;
  **5** avec trois ou quatre lignes — à quatre lignes (réserve 19 mm, plancher 139 mm), la cinquième ligne est à `ty = 139` **à
  l'égalité exacte** du plancher et passe (`ty < plancher` est faux) ; **4** avec cinq lignes (trois taux et arrondi, plancher
  143.5). Toutes ces grandeurs sont des multiples de 0,5, exactes en `f32`.
- `crates/kesh-api/src/routes/invoice_pdf_service.rs:235` `    if lines.len() > MAX_LINES_PER_PDF {` (`MAX_LINES_PER_PDF = 9`,
  `:95`) : un **pré-filtre** du rendu, avant toute géométrie, sur les **lignes de la pièce** ; `generate_qr_bill_pdf_with_date` ne
  le contient pas. Or un commentaire fait seulement de caractères sans glyphe (« Ω », U+200B) n'est pas vide pour la 49-1c
  (`validate_line`, `kesh-49-1c` `routes/invoices.rs:508-509`, `trim()` puis `is_empty()`) et s'enregistre, alors qu'il imprime
  **zéro** ligne (`lignes_de_commentaire`) : huit lignes chiffrées et deux tels commentaires font 8 lignes imprimées, 10 lignes de
  pièce (V1-1).
- `crates/kesh-api/src/routes/invoices.rs:931` `pub async fn validate_invoice_handler(` : appelle `invoices::validate_invoice`
  (`:940`, sous `retry_on_deadlock`) **sans aucun contrôle du PDF** ; c'est le seul appelant par une route (`kesh-seed/src/pieges.rs:590`
  valide par le dépôt, la démonstration). Il reçoit `validated.lines` (`:943-946`).
- `crates/kesh-api/src/routes/invoice_pdf_service.rs:39` `pub(crate) fn ligne_pdf(` et `:66` `pub(crate) fn vat_lines_pdf(` :
  les constructeurs du rendu, réutilisables par le handler (même crate).
- `crates/kesh-db/src/repositories/invoices.rs:2300-2301` : l'arrondi se fige à la validation par
  `kesh_core::accounting::vat::invoice_rounding(ttc_brut, settings.round_to_5_centimes)` ; l'aperçu d'un brouillon le refait par
  `routes/invoices.rs:332` `async fn with_draft_rounding_preview(` (lecture de `round_to_5_centimes`, `:340`, qui rend `true` à
  ligne absente, comme la validation).
- `invoice_pdf_service.rs:542` `        QrBillError::TooManyLines(n) => return AppError::InvoiceTooManyLinesForPdf(*n),` et
  `crates/kesh-api/src/errors.rs:2310` `pub(crate) fn message_trop_de_lignes_pdf(n: usize) -> String {` : le refus d'aujourd'hui,
  **au rendu**, sur une facture déjà validée (#1364), avec une phrase impossible à suivre sur une facture validée (#1110).
- Nuance à #1364 : une facture validée **se dévalide** (`kesh-db/src/repositories/invoices.rs:1637` `pub async fn unvalidate(`),
  sauf réglée, créditée, rappelée, envoyée, rapprochée ou sur un exercice clos (`:1617-1620`). Le défaut demeure : le refus arrive
  après que le numéro et l'écriture sont posés, et rien ne dit de dévalider.
- `frontend/src/routes/(app)/invoices/[id]/+page.svelte:448` `				validateError = err.message;` : l'écran affiche le message du
  serveur pour tout refus de validation, dialogue ouvert pour tout code non listé (`:1722-1727`, liste de fermeture `:490-497`,
  relu par la validation P1) : un refus neuf, localisé côté serveur, s'affiche sans changement de l'écran.
- `pdf.rs:756-765` : les conditions de paiement s'écrivent à `Mm(ty.max(content_floor + 5.0))` (`:762`) — elles se **tassent** au
  lieu de refuser (finding V1-13, D6).

## Inventaire par le symptôme

**S — « un refus du rendu PDF qui dépend du contenu figé de la facture tombe après sa validation »** (#1364), causes de
`QrBillError` triées (fiche de la 49-4, § S2, relu) :

- `TooManyLines` (lignes, récapitulatif) : contenu figé → **dans le périmètre** (D1).
- `InvalidCharset` / `FieldTooLong` sur le numéro de facture : contenu figé, porté par **#1046** ; `NUMBER_FORMAT_UNRENDERABLE`
  contrôle déjà la longueur à la validation (`docs/api-external.md:520`) → hors périmètre.
- `HeaderOverflow`, `InvalidIban`, `InvalidQrIban`, `InvalidCountry`, `FieldEmpty`, `InvalidCharset` sur noms et adresses : données
  de la société, du contact ou du compte, **corrigibles après la validation** → hors du symptôme.
- `InvalidAmount` : borné à la validation par le montant minimum ; la borne haute (999 999 999.99) n'est pas un cas réel → angle
  mort assumé.
- **Caractères sans glyphe** (#1400) : ils ne font pas refuser le rendu (ils disparaissent) → hors du symptôme, à la **49-4b**.
- **Rappel** : le contrôle ne compte pas le bloc de rappel (D2) ; une facture qui tient juste peut voir son rappel refusé au
  rendu → **limite déclarée au manuel**, avec sa sortie (avoir) ; remède : #1374 (v0.21.0).
- **Avoir** : son plancher (`CONTENT_FLOOR_NO_QR`) est plus bas que celui de la facture et il reprend au plus les lignes de sa
  facture : un avoir d'une facture contrôlée tient. Hors du symptôme.

## Décisions

- **D1 — La capacité se contrôle à la validation, par les mêmes paramètres que le rendu, et la garde du rendu reste l'oracle.**
  (Ex-D4 de la 49-4, réduite au dessin actuel, remédiée par V1-3.) `kesh-qrbill` extrait de `draw_invoice_section`, **sans changer
  le dessin**, deux fonctions privées que le dessin et le contrôle appellent tous deux : (i) `lignes_imprimees(&InvoiceLinePdf) ->
  usize` — **1** pour une ligne chiffrée, `lignes_de_commentaire(&description).len()` pour un commentaire ou un titre (0 possible) ;
  (ii) la **géométrie du tableau** — l'ordonnée de la première ligne (`PAGE_H - 130 - 5 - 3`), le pas (5 mm), le plancher
  (`content_floor + 15 + réserve du récapitulatif + bloc de rappel`). Une fonction **publique et pure** (sans document PDF),
  `controler_les_lignes(lignes: &[InvoiceLinePdf], vat_lines: &[InvoiceVatLinePdf], rounding: Decimal) -> Result<(),
  LignesHorsPage>` (noms au choix du développement ; `LignesHorsPage { besoin: usize, place: usize }`), compte les lignes imprimées
  et la place au plancher d'une **facture** (`SEP_Y`), **sans** bloc de rappel (D2), en **simulant la même suite d'opérations
  `f32`** que la boucle du dessin (soustractions successives de 5.0 depuis la même ordonnée, comparaison `ty < plancher`) : une
  formule de division serait juste aujourd'hui (grandeurs exactes), mais serait une seconde écriture de la boucle. Le
  récapitulatif se compte par `recap_lines` avec `QrBillI18n::default()` (seul le **nombre** compte, comme `reminder_block_height`) ;
  `subtotal_ht` n'en change pas le nombre (il suffit de le passer à zéro, ou de construire un `InvoicePdfData` minimal : choix du
  développement, signature écrite au Dev Agent Record).
  **La garde de `draw_invoice_section` reste la sienne** (`if ty < plancher_des_lignes`, avant chaque ligne) : il y a deux boucles,
  mais **une seule source** de ce qu'elles comptent (lignes imprimées, ordonnée, pas, plancher). C'est ce qui donne à T-PROP un
  oracle indépendant : le résultat réel de `generate_qr_bill_pdf_with_date` (`Ok` ou `Err(TooManyLines)`), non le plan comparé à
  lui-même (V1-3). Le dessin produit par la story est **octet pour octet** celui d'avant (AC2).
  `validate_invoice_handler` l'appelle **avant** `invoices::validate_invoice` : il lit le brouillon et ses lignes
  (`find_by_id_with_lines`, scopé société ; `None` ou un statut autre que `draft` laisse parler `validate_invoice`, qui rend son 404
  ou 409 habituel), construit les `InvoiceLinePdf` par `ligne_pdf` et les `vat_lines` par `vat_lines_pdf` (exactement comme
  `render_document`, `invoice_pdf_service.rs:428-441`), l'arrondi par `kesh_core::accounting::vat::invoice_rounding` sur
  `invoice_total_ttc` des paires `(line_total, vat_rate)` et le réglage `round_to_5_centimes` (comme la validation,
  `kesh-db/src/repositories/invoices.rs:2297-2301`), applique **d'abord le contrôle géométrique**, **puis le pré-filtre
  `MAX_LINES_PER_PDF` du rendu par la même fonction que `render_document`** (extraite dans `invoice_pdf_service.rs`, appelée par
  les deux : une seule source, aucune réponse du rendu ne change ; V1-1) — dans cet ordre, le refus du pré-filtre ne se lit que
  pour une pièce que le contrôle accepte (commentaires sans rien d'imprimable), où sa place de 9 est juste ; une pièce de douze
  lignes chiffrées dit « 12 » et « 8 », non « 9 » (V2-1) —, et refuse **sans rien écrire** (statut, numéro, écriture, version,
  compteur de numérotation inchangés). Le contrôle ne lit ni contact, ni compte bancaire, ni société : la validation n'acquiert
  aucun prérequis neuf. Une facture **dévalidée** qui se revalide y passe de même.
  **Course assumée** (V1-8) : le contrôle précède la transaction de validation ; un `PUT` concurrent du brouillon entre les deux
  peut faire valider des lignes non contrôlées ; la pièce rend alors au rendu le refus d'aujourd'hui et se dévalide tant qu'elle
  n'a été ni envoyée ni rappelée — borne réelle, mais que **rien ne signale** : issue à ouvrir (Dev Notes, (2)). *Écartées* : (a) le
  contrôle **dans** la transaction de `kesh-db` (un rappel passé à `validate_invoice`) — `kesh-db` deviendrait un module de plus,
  avec l'exception `kesh-db` du gate et une story plus lourde, pour une course qui exige deux onglets sur le même brouillon dans la
  même seconde ; (b) le contrôle au `PUT` du brouillon (il bloquerait la saisie d'un brouillon en cours) ; (c) un compteur dans
  l'écran (il dupliquerait la mesure AFM de `kesh-qrbill`, D12 de la 49-1c) ; (d) le PDF sur plusieurs pages (#1374, CR
  distincte) ; (e) faire du dessin un consommateur d'un plan calculé d'avance (ex-D4 de la 49-4) : un seul calcul, mais T-PROP
  deviendrait vrai par construction (V1-3), et le dessin changerait de structure dans une story qui ne doit pas le changer ;
  (f) un `tracing::warn!` après la validation, en rejouant le contrôle sur `validated.lines` (remède proposé par V1-8) — aucune
  infrastructure de test ne lit les journaux dans `kesh-api` (`grep -rln "traced_test\|logs_contain" crates/kesh-api` vide) :
  un avertissement non testé ; l'issue le trace mieux.
- **D2 — Le contrôle ne réserve pas le bloc de rappel ; limite déclarée au manuel ; plusieurs pages en v0.21.0 (décision de Guy
  du 2026-11-20, Q1 (c)).** Guy avait d'abord répondu « réserver » sur une estimation fausse de l'orchestrateur (« environ trois
  lignes ») ; la validation P1 (addendum A1) a établi que garantir le rappel demande de réserver le pire cas, 18 mm (règlement
  partiel et frais : quatre lignes), soit trois lignes imprimées seulement pour une facture à commentaires avec arrondi, aucune
  avec trois taux et arrondi. Guy a alors décidé : **aucune réserve** ; une facture (à commentaires ou non) qui tient juste peut
  voir son **rappel** refusé au rendu (`TooManyLines`) — **limite écrite au manuel**, avec la sortie (avoir) ; le remède est la
  facture sur plusieurs pages, **#1374, milestone v0.21.0**, qui couvre le rappel. Les points A1 à A4 de l'addendum de la
  validation P1 tombent avec cette décision, sauf ce qu'ils disent de la phrase du manuel (A3, joint à V1-3, D7). Le refus du
  rappel garde son code et sa phrase d'aujourd'hui (`INVOICE_TOO_MANY_LINES_FOR_PDF`, #1110).
- **D3 — Un code neuf pour le refus à la validation : `400 INVOICE_LINES_DO_NOT_FIT_PDF`.** (Ex-D5, phrase adaptée au dessin
  actuel.) La phrase dit ce qui est possible sur un brouillon : *« Les lignes de cette facture ne tiennent
  pas sur la page de son PDF : elles demandent { $besoin } lignes, la page en offre { $place }. Retirez des lignes,
  raccourcissez les commentaires, ou répartissez les lignes sur deux factures. La facture reste un brouillon. »* (clé
  `error-invoice-lines-do-not-fit-pdf`, quatre catalogues). `besoin` et `place` comptent les lignes imprimées ; pour le
  pré-filtre (V1-1), les lignes de la pièce et `MAX_LINES_PER_PDF` — d'où « lignes » sans « imprimées ». **Elle ne dit pas « raccourcissez des désignations »** : tant que la
  49-4b n'est pas livrée, une désignation est réduite à une ligne, la raccourcir ne libère rien ; la 49-4b réécrira la phrase.
  *Écartée* : reprendre `INVOICE_TOO_MANY_LINES_FOR_PDF` — documenté pour le **rendu** (`docs/api-external.md:1281`), sa phrase
  compte les **lignes de la pièce** (une facture de deux lignes dont un commentaire long dirait « contient 2 lignes »). Le refus
  au rendu garde son code et sa phrase : **#1110** reste ouverte, hors périmètre ; après cette story, elle ne se lit plus que pour
  une pièce validée avant la 0.20.0, un rappel (D2) ou la course (D1).
- **D4 — Aucun changement de l'écran ; un E2E le joue.** (Ex-D7.) Le dialogue de validation affiche le message du serveur
  (`+page.svelte:448`). Aucun fichier de `frontend/src` ne change ; un E2E Playwright joue le refus dans le navigateur (AC5).
  *Écartée* : une mise en évidence de la ligne fautive (écran neuf, `InvoiceForm.svelte`, commun avec 49-1c, 49-2, 49-3).
- **D5 — Aucune ligne de `kesh-db`, aucune écriture en base, aucun changement du dessin.** (Ex-D10, étendue.) `git diff
  "$BASE"..HEAD -- crates/kesh-db crates/kesh-core crates/kesh-api/src/routes/issued_invoice_pdf.rs
  crates/kesh-api/src/routes/credit_notes.rs crates/kesh-qrbill/src/largeurs.rs crates/kesh-qrbill/src/texte.rs` est vide
  (AC6). `validate_line` et `MAX_DESCRIPTION_LEN` (1000) ne changent pas : la saisie d'un brouillon reste libre ; seule la
  validation contrôle. Le pré-filtre `MAX_LINES_PER_PDF = 9` (`invoice_pdf_service.rs:95`, `:235`) garde sa valeur et sa réponse
  au rendu ; il est seulement extrait pour que la validation l'applique aussi (D1, V1-1). `$BASE` = `git merge-base HEAD
  origin/epic/49-types-de-lignes-facture`, **recalculée après chaque reprise** de la branche d'épique et écrite au Dev Agent
  Record (V1-2) : ainsi les lignes que la 49-1c ou la 49-3 apportent par la fusion ne comptent pas.
- **D6 — « Tient » veut dire « le rendu rend `Ok` », non « la page se lit » ; la recette le regarde.** (V1-13.) Au seuil, les
  conditions de paiement se tassent à `max(ty, content_floor + 5)` (`pdf.rs:762`), à environ 2 mm sous le total : comportement de
  la 0.19.0, que le contrôle ne change pas. La recette visuelle (AC7) capture une facture **au seuil avec conditions de
  paiement** ; si le texte chevauche, une issue s'ouvre (P3, comportement hérité), sans changer le contrôle.
- **D7 — Le manuel réécrit la phrase de capacité de la 49-1c après sa fusion.** (Ex-D9 ; V1-3 et A3.) La 49-1c écrit à
  `user-manual.tex:1456` (sur sa branche) qu'une facture tient « huit au plus, moins avec le récapitulatif de TVA » : faux pour
  Guy, non assujetti, l'arrondi aux 5 centimes étant actif par défaut (`company_invoice_settings.rs:760-772`) et donnant un
  récapitulatif (« Sous-total », « Arrondi ») sans aucune TVA. La 49-4a **réécrit** cette phrase, **après la fusion de la 49-1c**
  dans `epic/49-types-de-lignes-facture` et sa reprise ici (commit de fusion relu) : une facture tient **huit lignes imprimées au
  plus, moins avec un récapitulatif — TVA ou arrondi** ; **Kesh le vérifie à la validation** : une facture qui ne tiendrait pas
  est refusée en le disant, le brouillon reste modifiable ; **le rappel d'une facture qui remplit juste sa page peut être refusé
  faute de place** (D2) : la sortie est d'annuler la facture par un avoir et de la refaire sur deux factures. Si la 49-1c n'est
  pas fusionnée quand le reste de la story est clos, la 49-4a écrit la même phrase à l'item « Lignes de facture » de la saisie
  (`:1456` de la branche) et **le second arrivé réconcilie** (`CLAUDE.md` § *Fusions et fichiers communs*), averti au Dev Agent
  Record. Le guide de l'API porte la même phrase (AC8). La phrase de
  `:1594` (« disparaît du PDF sans avertissement », conseil du trait d'union) **reste vraie** après la 49-4a seule et **ne se
  touche pas** ici (V1-6 : c'était l'inverse dans le découpage d'origine).

## Critères d'acceptation

- **AC1 — La capacité se contrôle à la validation.** `POST /api/v1/invoices/{id}/validate` sur un brouillon :
  (a) d'une ligne chiffrée et d'un **commentaire de 1000 caractères** (créé **par le dépôt**, l'API ne créant pas de commentaire
  avant la 49-1c) ; (b) de **huit lignes chiffrées à deux taux de TVA** (récapitulatif de trois lignes, place 5) ; (c) de **sept
  lignes chiffrées sans TVA, arrondi aux 5 centimes activé et non nul** (récapitulatif « sous-total, arrondi », place 6) — la même
  pièce **passe** arrondi désactivé ; (d) de **neuf lignes chiffrées courtes** ; (e) de **huit lignes chiffrées sans récapitulatif et deux commentaires « Ω »** (sans
  glyphe, zéro ligne imprimée, créés par le dépôt ; pré-filtre, V1-1) ; (f) de **douze lignes chiffrées sans récapitulatif**
  (contrôle géométrique avant le pré-filtre, V2-1) — rend `400 INVOICE_LINES_DO_NOT_FIT_PDF`, message
  aux deux nombres justes ((e) : 10 et 9 ; (f) : 12 et 8). La pièce (e), validée par le dépôt, rend au `GET /pdf` `400
  INVOICE_TOO_MANY_LINES_FOR_PDF` : c'est bien le refus que la validation devance. Après le refus : `status = 'draft'`, `invoice_number` et `journal_entry_id` nuls, `version` inchangée,
  aucune ligne neuve de `journal_entries`, compteur de numérotation inchangé (la validation suivante d'une facture qui tient reçoit
  le numéro attendu).
- **AC2 — Le seuil exact passe, et le PDF se rend.** (a) Huit lignes chiffrées sans récapitulatif ; (b) cinq lignes chiffrées
  avec un récapitulatif de **quatre** lignes (seuil à l'égalité flottante) ; (c) cinq lignes chiffrées et un commentaire de trois
  lignes imprimées (créé par le dépôt), sans récapitulatif — la validation **réussit** et `GET /pdf` rend `200`. Une facture dévalidée puis allongée
  au-delà de la place est refusée à la revalidation, sans rien écrire.
- **AC3 — Un seul ensemble de paramètres, deux boucles qui s'accordent.** Pour une famille de pièces engendrées (déterministe,
  graine fixe, au moins 500 pièces : nombre de lignes chiffrées, commentaires de longueurs variées dont vides et faits de blancs,
  zéro à trois taux de TVA, arrondi nul ou non, et des cas au seuil exact de chaque taille de récapitulatif), `controler_les_lignes`
  rend `Ok` **si et seulement si** `generate_qr_bill_pdf_with_date` (avec un contact, un compte et un en-tête minimaux qui ne
  débordent pas) rend `Ok` ; et quand il rend `Err`, c'est `TooManyLines`. Le dessin garde sa propre garde (D1). Le pré-filtre
  `MAX_LINES_PER_PDF`, absent de `generate_qr_bill_pdf_with_date`, est hors de cette propriété : AC1 (e) le couvre par la route et
  `GET /pdf`. La famille porte les seuils à 8, 6, 5 et 4.
- **AC4 — Le PDF ne change pour aucune pièce.** Les tests avant / après de la 49-1b sont **verts sans modification** — S1 à S9,
  **S7 compris** (`crates/kesh-qrbill/tests/rendu_avant_apres.rs` : `neuf_pieces_chiffrees_se_rendent_comme_en_0_19_0`,
  `les_deux_coupes_de_la_qr_sont_inchangees`) et les pièces HTTP a à d, regel compris (`crates/kesh-api/tests/pdf_avant_apres_e2e.rs` :
  `quatre_pieces_chiffrees_se_rendent_comme_en_0_19_0`) ; `git diff "$BASE"..HEAD -- crates/kesh-qrbill/tests/
  crates/kesh-api/tests/fixtures/ crates/kesh-api/tests/pdf_avant_apres_e2e.rs crates/kesh-qrbill/src/empreinte.rs` ne nomme que des
  fichiers de tests **neufs** (aucun fichier existant de ces chemins modifié). Les tests de rendu existants (`invoice_pdf_e2e`,
  `invoice_frozen_pdf_e2e`, `invoice_send_email_e2e`, `dunning_reminders_e2e`, `golden_test`, `invoice_line_types_e2e`, tests de
  `pdf.rs`) sont verts sans modification d'assertion (T10).
- **AC5 — E2E.** `frontend/tests/e2e/invoice-validation-pdf.spec.ts` : (a) un brouillon de **neuf lignes chiffrées** (créé par
  l'API dans le montage) → *Valider* → le message du refus de capacité est **visible** dans le dialogue, avec les deux nombres ; la
  facture reste « Brouillon », sans numéro ; (b) après le retrait d'une ligne, la validation réussit et le téléchargement du PDF
  répond `200`. Le smoke de la CI n'est pas modifié.
- **AC6 — Factures et avoirs existants inchangés, aucune écriture.** Le `git diff` de D5 est vide ; les lignes, totaux, TVA,
  écritures, numéros d'une facture validée ne bougent sur aucun chemin ; le PDF archivé est servi tel quel (T12 de la 49-1b vert
  sans modification) ; une facture validée avant la 0.20.0 qui ne tiendrait pas n'est concernée par rien (le contrôle ne s'applique
  qu'à la validation) ; `kesh-seed` valide par le dépôt et n'est pas concerné.
- **AC7 — Recette visuelle** (`CLAUDE.md` § *Recette visuelle*) : captures PNG, thème clair, sur une base de démonstration
  (`kesh_e2e_e494a`, jamais la base du NAS) — le dialogue de validation sur le refus de capacité ; le PDF (`mutool draw -r 150`)
  d'une facture **au seuil exact avec conditions de paiement** (D6), et d'une facture au seuil avec récapitulatif. Dans
  `kesh-gate-logs/e494a/captures/`, jointes au Dev Agent Record et au prompt de revue ; chaque capture se **lit** (message entier
  et lisible ; conditions de paiement sans chevauchement, sinon issue ouverte).
- **AC8 — Textes et contrat.** Un code neuf (`INVOICE_LINES_DO_NOT_FIT_PDF`, `400`) ; sa clé dans les **quatre** catalogues
  (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`, bloc de l'Epic 49 à créer, ou celui de la 49-1c après fusion) et dans la garde
  `crates/kesh-api/tests/textes_coherents.rs` (même forme que `error-invoice-too-many-lines-for-pdf`) ; `docs/api-external.md` :
  la route de validation (`:520`) nomme le refus, en **changement de contrat** (une facture que la 0.19.0 validait peut être
  refusée), la table des codes (`:1281`) gagne une ligne et dit que `INVOICE_TOO_MANY_LINES_FOR_PDF` ne se rencontre plus que pour
  une pièce validée avant la 0.20.0, un rappel (D2) ou la course (D1) ; la phrase de capacité (« huit lignes au plus, moins avec un
  récapitulatif — TVA ou arrondi ») et la limite du rappel avec sa sortie par avoir y figurent comme au manuel (D7) ;
  `admin-manual.tex` § *Pour les intégrations* de `sec:maj-0-20-0` (`:2559`) gagne une phrase sur ce refus, en changement de
  contrat.
- **AC9 — Manuel utilisateur** (D7) : la phrase de capacité, **réécrite**, dit « huit au plus, moins avec un récapitulatif — TVA
  ou arrondi », que Kesh la vérifie à la validation et que le brouillon reste modifiable, et la limite du rappel avec sa sortie par
  avoir. **Contrôles** (sortie copiée au Dev Agent Record) : `git grep -n -i "récapitulatif" -- docs/manual/fr/user-manual.tex`
  nomme la phrase neuve, et cette ligne contient « arrondi », « validation », « rappel » et « avoir » ; la même chose sur
  `docs/api-external.md` ; `git grep -n "récapitulatif de TVA}" -- docs/manual/fr/user-manual.tex` est vide (l'ancienne forme) ; `git grep -n -i "trait d'union ordinaire\|sans avertissement" -- docs/manual/fr/user-manual.tex` rend **la même
  sortie** qu'à la tête de départ (ou que la 49-1c fusionnée) : la 49-4a n'y touche pas. Le PDF du manuel n'est pas commité ;
  `pdftotext` du PDF régénéré localement, aplati (`tr '\n' ' ' | tr -s ' '`), relu sur la phrase neuve.
- **AC10 — Coordination.** La story ne touche ni `crates/kesh-qrbill/src/largeurs.rs`, ni `texte.rs`, ni `kesh-report/`, ni
  `CHANGELOG.md`, ni `frontend/src/`. `lib.rs` de `kesh-qrbill` ne change que par l'ajout de `controler_les_lignes` et de son type
  au ré-export (`lib.rs:24-31`), dans un hunk distinct de celui de #1327 (`pub use largeurs::largeur_mm;`, ligne voisine : relire à
  la fusion). `sprint-status.yaml` : seul l'orchestrateur le tient.
- **AC11 — Gates.** § *Tâches*, dernière tâche ; le Dev Agent Record n'affirme que ce qui a tourné.

## Tests à écrire, et la preuve du rouge

Journaux de rouge dans `/home/gcorbaz/devel/kesh-gate-logs/e494a/` (`rouge-T<n>[-m].log`), commande et sortie. Chaque mutation
s'**ajoute au code**, le test tourne, le journal se garde, la mutation se retire **et le fichier se touche** (`touch`) avant la
reconstruction (cargo garde sinon le binaire muté).

| # | Test | Ce qu'il prouve | Preuve du rouge |
|---|---|---|---|
| T1 | `kesh-api/tests/invoice_validation_pdf_e2e.rs::la_validation_refuse_une_facture_qui_ne_tient_pas` (AC1, AC2) | refus 400 sur (a) à (f), le refus au rendu (`GET /pdf`) de la pièce (e) validée par le dépôt, rien d'écrit (statut, numéro, écriture, version, compteur), seuils (a) à (c) acceptés et PDF `200`, l'arrondi compté, la revalidation contrôlée | mutations : (1) le handler n'appelle pas le contrôle → **ROUGE** ; (2) le contrôle ignore l'arrondi → **ROUGE** sur (c) ; (3) le contrôle compte une ligne par ligne de la pièce (sans les lignes du commentaire) → **ROUGE** sur (a) ; (4) le contrôle compare `ty <= plancher` → **ROUGE** sur AC2 (b) ; (5) le handler n'applique pas le pré-filtre → **ROUGE** sur AC1 (e) (`rouge-T1-1..5.log`) |
| T2 | `pdf.rs::tests::le_controle_et_le_rendu_s_accordent` (AC3) | équivalence `controler_les_lignes` ⇔ `generate_qr_bill_pdf_with_date` `Ok`, sur la famille engendrée ; oracle = la garde du **dessin**, non le plan | mutations : (1) la garde du dessin passe à `ty < plancher - 0.5` (le dessin admet une ligne de plus) → **ROUGE** ; (2) le contrôle part d'une ordonnée de 160 au lieu de la géométrie partagée (copie divergente simulée) → **ROUGE** ; (3) le contrôle seul compare `ty <= plancher` → **ROUGE** aux seuils à l'égalité (`rouge-T2-1..3.log`) |
| T3 | tests de la 49-1b, `rendu_avant_apres.rs` et `pdf_avant_apres_e2e.rs`, **non modifiés** (AC4) | S1 à S9 (S7 compris), a à d identiques : le dessin n'a pas bougé | mutation : l'ordonnée de la première ligne, dans la géométrie partagée, passe de 159 à 158.5 mm → **ROUGE** sur S1 (ordonnées décalées) ; mutation 2 : la ligne chiffrée dessinée par `truncate_display(…, 44)` → **ROUGE** sur S7 (`rouge-T3-1..2.log`) |
| T4 | `textes_coherents.rs` (AC8), entrée neuve pour `error-invoice-lines-do-not-fit-pdf` | la clé existe dans les quatre catalogues, avec `$besoin` et `$place` | mutation : la clé retirée du catalogue `it-CH` → **ROUGE** (`rouge-T4.log`) |
| T5 | contrôle `git diff` de D5 et AC4, script en lecture seule `kesh-gate-logs/e494a/controles-diff.sh` (AC4, AC6) | rien de `kesh-db`, `kesh-core`, `issued_invoice_pdf.rs`, `credit_notes.rs`, `largeurs.rs`, `texte.rs` ; aucun test ni fixture existant modifié | sonde positive : le script, sur `"$BASE"..HEAD`, lancé contre un commit qui touche `texte.rs` (worktree jetable ou `git diff` d'un commit fabriqué hors branche) rend une sortie non vide (`rouge-T5.log`) |
| T6 | E2E `frontend/tests/e2e/invoice-validation-pdf.spec.ts` (AC5) | (a) et (b) dans le navigateur | rouge constaté en lançant le fichier sur le binaire **sans** le contrôle (handler muté, binaire reconstruit) : (a) échoue (`rouge-T6.log`) |
| T10 | épreuve, non test neuf : `invoice_pdf_e2e`, `invoice_frozen_pdf_e2e`, `invoice_send_email_e2e`, `dunning_reminders_e2e`, `golden_test`, `invoice_line_types_e2e`, tests de `pdf.rs` | verts **sans modification d'assertion**. Un test existant qui valide **par la route** une pièce qui ne tient pas devient rouge par conception : il se liste au Dev Agent Record avec le changement fait (validation par le dépôt, ou pièce raccourcie) et sa raison. Cas relu : `invoice_pdf_e2e.rs::pdf_d_une_facture_a_commentaire_l_imprime` (`:1032`, commentaire de mille caractères) valide **par le dépôt** (`invoice_pdf_e2e.rs:215`, `kesh_db::repositories::invoices::validate_invoice`) : il reste vert et continue de prouver le refus **au rendu** d'une pièce validée sans contrôle | — |

## Tâches

- [ ] **Étape 0 — relire.** (1) Les citations de « Ce qui se lit au code », à la tête ; `git log --oneline
  HEAD..origin/epic/49-types-de-lignes-facture -- crates/kesh-qrbill crates/kesh-api/src/routes` : si la branche d'épique a bougé
  (49-1c, 49-3), la reprendre d'abord (commit de fusion relu) ; puis calculer `$BASE` (D5) et l'écrire au Dev Agent Record — de
  même après chaque reprise. (2) Refaire l'arithmétique des seuils (8, 6, 5, 4, et l'égalité à
  quatre lignes de récapitulatif) sur le code, la consigner au Dev Agent Record. (3) Vérifier que `validate_invoice` accepte un
  brouillon à commentaire créé par le dépôt (49-1a), sinon adapter le montage d'AC1 (a) et le dire.
- [ ] **`kesh-qrbill`** : `lignes_imprimees`, la géométrie partagée, `controler_les_lignes` et `LignesHorsPage` (D1) ;
  `draw_invoice_section` lit la géométrie partagée et `lignes_imprimees` **sans changer un octet de sa sortie** ; doc-comments
  (`draw_invoice_section`, `controler_les_lignes`) ; ré-export dans `lib.rs`. **Ne pas** toucher `largeurs.rs`, `texte.rs`,
  `truncate_display`, `lignes_de_commentaire`.
- [ ] **`kesh-api`** : dans `validate_invoice_handler`, avant `invoices::validate_invoice` : lecture du brouillon, contrôle (D1),
  puis pré-filtre extrait de `render_document` et appelé par les deux ; le doc-comment de `MAX_LINES_PER_PDF`
  (`invoice_pdf_service.rs:85-95`, qui dit la garde « renvoie `QrBillError::PdfGeneration` » et décrit un seuil « `0` sans TVA »)
  se réécrit sur la fonction extraite : la garde rend `TooManyLines` et réserve aussi pour l'arrondi (V2-4) ;
  une variante d'`AppError` (`InvoiceLinesDoNotFitPdf { besoin, place }`, à côté de `InvoiceTooManyLinesForPdf`, `errors.rs:489`) et
  sa réponse, phrase par `t_args` ; rien d'autre dans `routes/invoices.rs` (la 49-1c y touche `validate_line` et la garde : hunks
  distincts).
- [ ] **`kesh-i18n`** : une clé × quatre catalogues, bloc de l'Epic 49 ; `textes_coherents.rs` (une entrée).
- [ ] **Tests** T1 à T6, avec leurs rouges ; T10 relu.
- [ ] **Documents** (AC8, AC9, D7) : `docs/api-external.md`, `admin-manual.tex` (`sec:maj-0-20-0`), puis — après la fusion de la
  49-1c, ou selon D7 — `user-manual.tex` ; contrôles de l'AC9, sortie copiée.
- [ ] **E2E et captures** (AC5, AC7) : le fichier Playwright, son rouge (T6), les captures relues.
- [ ] **Gates** (voie A, base et Rust ; aucune ligne de `kesh-db` : pas d'exception `kesh-db`). En cours de boucle, gate ciblé :
  `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run -E 'package(kesh-qrbill)
  | binary(kesh_api) | binary(pdf_avant_apres_e2e) | binary(invoice_validation_pdf_e2e) | binary(invoice_pdf_e2e) |
  binary(invoice_frozen_pdf_e2e) | binary(invoice_send_email_e2e) | binary(dunning_reminders_e2e) | binary(invoice_line_types_e2e) |
  binary(textes_coherents)'` ; frontend, parce que les catalogues changent : `npm run check`, `npm run lint-i18n-ownership`,
  `npm run test:unit` (sous `mem-guard`). **Au passage en `review`** : `scripts/test-fast.sh --ci` puis `scripts/gate-ci.sh
  kesh_ci_e494a`, bases `kesh_e494a` remises à neuf (`CLAUDE.md` § *Base de gate*, sans redémarrer le conteneur). **À la clôture**
  (dernier commit de code) : `gate-ci` complet et **E2E Playwright complet** sur `kesh_e2e_e494a` (montage `docs/testing.md`
  § *Prérequis Playwright local*), « vert » = les sept KF-029 seulement, jugés fichier par fichier. Une commande lourde à la fois,
  `scripts/mem-guard.sh --status` avant chacune, `flock` du verrou de la voie.
- [ ] **Contrôle des mots-clés** : `git log origin/epic/49-types-de-lignes-facture..HEAD --format=%B | /usr/bin/grep -niE
  '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` vide.

## Note de version

*(Texte pour le `CHANGELOG`, assemblé à l'intégration ; la story n'écrit pas `CHANGELOG.md`.)*

- **Factures — contrôle à la validation** : une facture dont les lignes ne tiendraient pas sur la page de son PDF est refusée à
  la validation, en le disant (`400 INVOICE_LINES_DO_NOT_FIT_PDF`, avec le nombre de lignes demandées et offertes),
  quand le brouillon se corrige encore, au lieu d'être validée puis refusée au premier téléchargement (`refs #1364`).
  **Changement de contrat** pour les intégrations : une facture que la 0.19.0 validait sans en tirer de PDF peut être refusée.
  Aucune facture, aucun avoir, aucun rappel existant ne change ; le PDF n'est pas modifié. Le rappel d'une facture qui remplit
  juste sa page peut être refusé faute de place : limite écrite au manuel, avec la sortie par un avoir.

## Dev Notes

- **Findings de la validation P1 de la 49-4** (`kesh-gate-logs/e494/validation-49-4-p1.md`), relus au code le 2026-11-20 à
  `a1877ef9f`, et leur sort ici :
  - **V1-3 (MEDIUM)** — T-PROP vrai par construction : **remédié** par D1 (le dessin garde sa garde, la géométrie et le compte
    des lignes ont une seule source, T2 interroge le rendu réel). Prémisse relue : `pdf.rs:644-665`, garde dans la boucle du
    dessin. La moitié T-SRC du finding va à la 49-4b.
  - **V1-5 (MEDIUM)** — rappel d'une facture qui tient juste : **tranché par Guy le 2026-11-20** (D2) — aucune réserve, limite
    écrite au manuel avec sa sortie par avoir, remède #1374 (v0.21.0). Prémisse relue : `pdf.rs:631-645` (réserve du rappel dans le rendu), `reminder_block_height` `pdf.rs:1998-2004`.
  - **V1-6 (MEDIUM)** — découpage : **remédié** par le découpage lui-même (cette story = D4 + D5 + D7 de la 49-4 sur le dessin
    actuel, aucune pièce existante ne change).
  - **V1-8 (LOW)** — course non signalée et argument de splitting inexact : argument **reformulé** (D1, écartée (a)) ; la course
    **tracée** par une issue (ci-dessous, (2)) ; le `warn!` écarté (D1 (f)). Prémisse relue : `routes/invoices.rs:931-946`, aucun
    verrou ni version.
  - **V1-10 (LOW)** — citations : celles qui servent ici sont relues et justes (`invoice_pdf_e2e.rs:1032` pour la fonction, non
    `:1030`) ; les autres (`credit_notes.rs:364-365`, `types.rs:196`, `pdf.rs:318`) vont à la 49-4b.
  - **V1-11 (LOW)** — fichiers communs avec la 48 : **remédié** (ci-dessous). Prémisse relue : `epic-48-corrections-0-20-0.md:201-202`
    nomme `errors.rs`, `docs/api-external.md` et les manuels pour la 48-1 et la 48-2.
  - **V1-12 (LOW)** — renvois AC ↔ tests : **remédié** pour ce qui est d'ici (AC8 → T4, AC4/AC6 → T5) ; le reste (grec d'une pièce
    d'avant, mutation de T5, invisibles) va à la 49-4b.
  - **V1-13 (LOW)** — conditions de paiement au seuil : **remédié** par D6 et AC7 (capture au seuil, issue si confirmé).
  - **V1-1, V1-2, V1-4, V1-7, V1-9** : hors de ce périmètre (rendu des désignations, repli, dérogation) → **49-4b**.
- **Issues à ouvrir** (demandées à l'orchestrateur, une par problème) : (1) *sans objet* : la limite du rappel est
  écrite au manuel et son remède est #1374 (D2) ; (2) *Validation d'une facture : le contrôle de capacité du PDF précède la
  transaction ; une modification concurrente du brouillon n'est pas signalée* (`technical-debt`, P4) ; (3) *PDF de facture : au
  seuil de capacité, les conditions de paiement chevauchent le total* — **seulement si** la recette (AC7) le montre (P3).
- **Fichiers communs avec les autres stories de la release** :
  - `crates/kesh-qrbill/src/pdf.rs` : la **49-4b** (même bloc de lignes, après la 49-4a), la **49-3** (rendu en évidence du titre :
    elle passera par `lignes_imprimees`).
  - `crates/kesh-api/src/routes/invoices.rs` : la **49-1c** (`validate_line`, `verify_vat_rates_against_db`), la **49-3**
    (garde `heading`), la **49-4b** (contrôle des caractères, dans le même handler) ; la 49-4a n'y touche que
    `validate_invoice_handler`.
  - `crates/kesh-api/src/routes/invoice_pdf_service.rs` : la **49-4a** (extraction du pré-filtre `MAX_LINES_PER_PDF`, doc-comment
    réécrit) puis la **49-4b** (`CaractereRefuse`, `:646-650`) ; la 49-4b part de la 49-4a (V2-4).
  - `crates/kesh-api/src/errors.rs` : la **48-1** et la **48-2** (`epic-48 :201-202`), la **49-4b** ; variantes et fonctions
    ajoutées, le second arrivé fait l'union.
  - `docs/api-external.md` : 48-1, 49-1c (`:512`), 49-3, 49-4b ; la 49-4a touche `:520` et la table des codes (`:1281`).
  - `docs/manual/fr/admin-manual.tex` § `sec:maj-0-20-0` : 48-1, 49-1a, 49-1c, 49-1d, 49-4b ; `user-manual.tex` : 48-1, 49-1c (D7),
    49-4b ; la PR #1327 (`:4082`, `:4181`) : distinct.
  - Catalogues i18n ×4 : toutes les stories ; bloc de l'Epic 49.
- **PR #1327 de Beat** (lecture seule) : recoupe `kesh-qrbill/src/lib.rs` (hunk voisin, sans contradiction) ; la 49-4a ne touche ni
  `largeurs.rs` ni le `README.md` de la crate.
- **Splitting** (`CLAUDE.md`, au grain de la crate) : `kesh-qrbill`, `kesh-api`, `kesh-i18n`, documents (`docs/manual/fr`,
  `docs/api-external.md`) — **quatre modules**, plus des tests (Rust et un fichier Playwright, aucun fichier de `frontend/src`).
  Sous le plafond.
- **Charge** (estimation) : étape 0 un quart de journée ; `kesh-qrbill` une demi-journée ; `kesh-api` et i18n une demi-journée ;
  tests et mutations une demi-journée à un jour ; documents, E2E, captures une demi-journée ; gates une demi-journée — **deux et
  demi à trois jours** (le pré-filtre de V1-1 et la phrase réécrite de D7 n'y ajoutent qu'une heure ou deux), plus la validation et la boucle de revue. Tenable pour une clôture le jeudi 2026-11-26 au soir si la
  validation se lance le 2026-11-20. **En cas de manque de temps, la règle est la réduction de l'epic (`CLAUDE.md` § *Cadence*),
  jamais le rognage des gates** (V2-5).
- **Ce qui doit rester vrai** : (1) aucun PDF ne change ; (2) aucune pièce d'avant ne devient impossible à imprimer (le contrôle
  ne vaut qu'à la validation) ; (3) la validation n'exige rien de plus qu'aujourd'hui pour une facture qui tient et s'imprime ;
  (4) aucune ligne de `kesh-db`.
- **Pièges** : (a) la place se calcule en simulant la boucle en `f32`, comme le dessin (une seule écriture de la boucle) ; (b) `recap_lines`
  avec `QrBillI18n::default()` ne sert qu'à compter ; (c) l'arrondi du contrôle se décide au centime comme `recap_lines`
  (`pdf.rs:1898-1903`) ; (d) un commentaire sans rien d'imprimable compte **zéro** ligne, comme au dessin (`lignes_de_commentaire`) ;
  (e) le refus porte sur la facture, sans nommer de ligne : le besoin et la place suffisent ; (f) l'avoir n'est pas contrôlé (il
  n'a pas de validation de ce type, et tient dès que sa facture tient, plancher plus bas) ; (g) le pré-filtre compte les lignes
  **de la pièce**, le contrôle les lignes **imprimées** : la validation applique les deux (V1-1).
- **Aucune donnée réelle** de Guy dans la fiche, les tests, les captures ou les journaux : tout est synthétique. `~/kesh/` n'est ni
  lu ni écrit.
- **Références** : fiche d'origine `49-4-designation-longue-pdf.md` (statut `split`) ; fiche sœur
  `49-4b-designation-longue-retour-ligne.md` ; `epic-49-types-de-lignes-facture.md` (`:130`, `:159` dans `kesh-e49`) ; `release-0.20.0.md` ;
  `49-1b-types-de-lignes-pdf.md` (D6) ; `49-1c-types-de-lignes-ecran.md` (D12) ; #1364, #1110, #1374, #1046, #1327 ;
  `kesh-gate-logs/prompts/e0200-commun.md`, `e0200-fiche.md` ; validation P1 de la 49-4.

## Dev Agent Record

### Base du diff (D5, V1-2)

- 2026-11-20, étape 0 : `git fetch origin` ; `git log --oneline HEAD..origin/epic/49-types-de-lignes-facture` vide (la branche
  d'épique n'a pas bougé depuis la fusion de la 49-1b) ; **`$BASE` = `git merge-base HEAD origin/epic/49-types-de-lignes-facture`
  = `f57f6ac80`** (« Merge story/49-1b-types-de-lignes-pdf dans epic/49-types-de-lignes-facture »). Aucune reprise.

## Change Log

| Date | Entrée |
|---|---|
| 2026-11-20 | **Création par découpage de la 49-4** (Opus 5.5 ; décision de Guy du 2026-11-20, sur la recommandation de l'orchestrateur du 2026-11-19 et le finding V1-6 de la validation P1). Reprend D4, D5, D7, D10 de la 49-4 sur le **dessin actuel** (aucun PDF ne change). Findings P1 remédiés ici, prémisse relue au code à `a1877ef9f` : V1-3 (le dessin garde sa garde, géométrie et compte des lignes à source unique, T2 interroge le rendu réel), V1-6 (le découpage), V1-8 (argument reformulé, course tracée par issue), V1-11 (fichiers communs avec la 48), V1-12 (renvois d'AC8 et AC6), V1-13 (capture au seuil avec conditions de paiement) ; V1-5 déclaré en D2, arbitrage à Guy par la question Q1 de la 49-4b. Phrase du refus sans « raccourcissez des désignations » (vraie seulement avec la 49-4b). Onze AC, six tests et une épreuve. Quatre modules. Aucune migration. Statut `backlog` : sa propre passe de validation est à venir. |
| 2026-11-20 | **Validation P1 remédiée** (Opus 5.5, prémisses relues au code ; rapport `kesh-gate-logs/e494/validation-49-4a-p1.md`). Tendance : P1 **0/0/3/9** ; addendum (D2 « réserver ») **0/0/3/1**, **tombé** par la décision de Guy du 2026-11-20 (D2 : aucune réserve, limite au manuel avec sortie par avoir, #1374 en v0.21.0) — A1, A2, A4 sans objet, A3 joint à V1-3. **MEDIUM** : V1-1 corrigé (pré-filtre `MAX_LINES_PER_PDF` extrait et appliqué à la validation, une seule source ; AC1 (e) « 8 chiffrées + 2 commentaires « Ω » » par la route et `GET /pdf`, mutation T1-5 ; prémisse : `invoice_pdf_service.rs:235`, `validate_line` de la 49-1c `:508-509` accepte « Ω ») ; V1-2 corrigé (`$BASE` = `merge-base` avec la branche d'épique, recalculée à chaque reprise, au Dev Agent Record : D5, AC4, T5, étape 0) ; V1-3 + A3 corrigés (phrase **réécrite** « moins avec un récapitulatif — TVA ou arrondi » + limite du rappel et sa sortie, au manuel et au guide de l'API ; AC9 grepe ces mots). **LOW** : V1-4 (justification de la simulation, mutation T2-3), V1-5 (AC8 aligné sur D3), V1-6 (`invoice_total_ttc`, `None`), V1-7 (seuil 4), V1-9 (lieu sans la 49-1c), V1-10 (changement de contrat), V1-11 (citations), V1-12 (montage d'AC2 (c)) corrigés d'une phrase ; V1-8 **tombé par décision**. Reclassements : aucun. D2 propagée (inventaire, D1, D3, D7, AC8, AC9, Dev Notes, issue (1) sans objet, Note de version). Phrase du refus : « lignes » sans « imprimées » (le pré-filtre compte les lignes de la pièce). Charge inchangée, deux et demi à trois jours. |
| 2026-11-20 | **Validation terminée** (passe P2 Sonnet, contexte frais, rapport `kesh-gate-logs/e494/validation-49-4a-p2.md` : **0/0/0/5**, aucun finding au-dessus de LOW). Tendance : P1 Opus **0/0/3/9** (addendum 0/0/3/1, **tombé par décision de Guy du 2026-11-20** : D2, aucune réserve du bloc de rappel) → P2 Sonnet **0/0/0/5**. Modèles : Opus → Sonnet. Reclassements : aucun. LOW de la P2 traités à l'étape 0 du développement (Opus 5.5) : **V2-1** (D1 : le contrôle géométrique **avant** le pré-filtre ; AC1 (f), douze lignes chiffrées, « 12 et 8 » ; ligne lue : `invoice_pdf_service.rs:235` `    if lines.len() > MAX_LINES_PER_PDF {`) ; **V2-2** (T1 « (a) à (f) » et le refus au rendu de la pièce (e) ; `grep "(a) à (d)"` vide après correction) ; **V2-3** (fiche 49-4b : volet Q1 (c) retiré des « Questions ouvertes », tranché à son § *Réponses de Guy*) ; **V2-4** (`invoice_pdf_service.rs` aux fichiers communs ; doc-comment de `MAX_LINES_PER_PDF` à réécrire dans la tâche `kesh-api` ; ligne lue : `invoice_pdf_service.rs:95` `pub const MAX_LINES_PER_PDF: usize = 9;`, et `:91-92` « la garde renvoie `QrBillError::PdfGeneration` ») ; **V2-5** (Dev Notes, *Charge* : réduction de l'epic, jamais rognage des gates). Statut `ready-for-dev`, puis `in-progress`. |
