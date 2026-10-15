# Prompt — revue de code P1, Story 30-2b (la création côte à côte, fournisseur proposé ou créé en ligne)

*Versionné le 2026-10-15. Trois lentilles (B, E, A), contexte frais chacune, lecture seule. Modèle : **Sonnet**
(passe complète — jamais Haiku pour une passe complète, D6 de la rétrospective de l'Epic 25).*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2b`, **reportée sur `fc3a4f7d`** (merge de la
30-1b, PR #690 ; la 30-2a est mergée en `f1508653`). Langue du rapport : français.

**Le diff à revoir** : `git -C /home/gcorbaz/devel/kesh-30-2 diff fc3a4f7d 1ff20e04 -- . ':(exclude)_bmad-output'` —
`1ff20e04` = **dernier commit de code** (backend, frontend, i18n, tests, E2E). Les commits qui le suivent ne portent que
la documentation (manuels et leurs PDF, CHANGELOG, README, `docs/api-external.md`), le registre, la fiche,
`sprint-status.yaml` et ce prompt : la documentation se lit **au HEAD** (`git diff fc3a4f7d HEAD -- docs CHANGELOG.md
README.md`). ⚠️ Pathspec `':(exclude)…'`, **non** `':!…'`.

Fiche : `_bmad-output/implementation-artifacts/30-2-documents-factures-fournisseur.md` — **partie b seulement** :
AC10 à AC17 et AC12-bis ; tâches T-b1 à T-b7 ; § *Tests*, *Gardes existantes* (lignes « b »), *E2E de parcours* (E1, E2,
E4), *Mutations* (M-b1 à M-b13), *Angles morts assumés* ; et le **Dev Agent Record** de la 30-2b (tests rouges,
mutations, gates, écarts, conflits). La 30-2c (AC18 à AC25 : `sourceDocument`, joindre, détacher, écriture) **n'est pas**
l'objet de la revue : un manque qui lui revient n'est pas un finding (vérifie-le à la fiche avant de l'écrire). Choix :
`epic-30-choix-autonomes-302.md`, C-30-2-1 à 29 (fiche et 30-2a) et **C-30-2-30 à 40** (développement de la 30-2b).
Règles : `CLAUDE.md` du dépôt — § *Un appariement automatique propose, il ne crée jamais*, § *Pattern batch*,
§ *Recompter ses propres comptes rendus*, § *Propagation post-patch*, § *Le prompt d'une passe doit NOMMER le manuel*,
§ *Une passe qui ne déclare pas ses axes exercés ne compte pas*. Issue : #685 — **aucun mot-clé de fermeture** dans un
message de commit, même en prose : `git -C /home/gcorbaz/devel/kesh-30-2 log fc3a4f7d..HEAD --format=%B | grep -niE
"\b(close[sd]?|fix(e[sd])?|resolve[sd]?):?\s+#[0-9]+"` doit être vide.

⛔ **Lecture seule stricte.** N'écris, ne modifie, ne crée aucun fichier hormis ton rapport. **Aucun** `git checkout`,
`git switch`, `git stash`, `git reset`, `git commit`, `git tag`, `git push`, `git rebase` — sous aucune forme. **Aucune
commande qui écrit, compile ou exécute** : ni `cargo`, ni `npm`/`npx`, ni `make`/`latexmk`, ni
`scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `python3`, ni `node`, ni
`sed -i`, ni `perl`. Aucune commande `gh` qui écrit (`gh issue view` est permis). Permis : `git diff`, `git show`,
`git log`, `git grep`, `grep`, `sed -n`, `cat`, `ls`, `jq`, et `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`
**vers la sortie standard uniquement** — jamais vers un fichier.

⛔ **Données réelles — dépôt public.** N'ouvre rien hors du worktree et de `/home/gcorbaz/devel/kesh-gate-logs/`. Les
captures `kesh-gate-logs/302b-captures/*.png` montrent des fixtures synthétiques : tu peux les ouvrir.

## Axes communs aux trois lentilles (chacune les déclare exercés ou non)

1. **La conversion commune et la validation** (`crates/kesh-api/src/routes/supplier_invoices.rs` :
   `normalize_payment_field`, `present`, `validate_payment_details`, `PaymentSource`,
   `CreateSupplierInvoiceRequest::to_new_supplier_invoice`) : une **seule** définition serveur pour les deux routes ?
   Les sept motifs, leur ordre et leur champ (C-30-2-31) ; l'ensemble de blancs retiré (`White_Space` + U+200B + U+FEFF)
   identique au test Vitest de `iban.ts` ; `Iban::new` (table des pays, longueur par pays, MOD-97), `QrIban::new` et la
   distinction `INVALID_IBAN` / `NOT_A_QR_IBAN` sur `creditorQrIban` (`CoreError::InvalidIban` propagé par
   `QrIban::new`) ; la référence : QRR normalisée puis `validate_qrr` avec un QR-IBAN, texte rogné ≤ 64 **caractères**
   sinon, 27 chiffres admis ; une chaîne vide ou blanche vaut absente ; valeurs stockées normalisées. Un corps peut-il
   encore faire enregistrer une coordonnée non validée (champ oublié, chemin de complétion qui contourne) ? Une
   référence de 64 caractères dont `trim` change la longueur ? `expectedPaymentAmount` (angle mort écrit) ?
2. **La complétion** (`crates/kesh-api/src/routes/imported_supplier_invoices.rs`, `complete_import_once`) : corps
   `CreateSupplierInvoiceRequest` ; avec QR, la source `Document` est construite **après** le verrou
   (`find_by_id_scoped_for_update`) et `QR_FIELDS_FROM_DOCUMENT` refuse les quatre champs ; le routage IBAN ⇔ QRR et
   `AMOUNT_MISMATCH` (sur `new.lines`) inchangés ; sans QR, `PaymentSource::Body` ; `project_id` honoré ;
   `CompleteImportRequest` disparu ; la réponse relue par `load_with_settlement_cancellation` **après** le `COMMIT`
   (C-30-2-33) — que se passe-t-il si la relecture échoue alors que la facture est créée (réponse d'erreur sur un
   succès) ? Le rejeu sur interblocage (`retry_app_on_deadlock`) voit-il toujours le même corps ? Les nouvelles
   variantes `AppError::InvalidPaymentDetails` et `QrFieldsFromDocument` (`errors.rs`) : statut, code, `details`,
   message par clé `error-…` présente dans les **quatre** locales.
3. **`supplier-candidates`** (route, `crates/kesh-db/src/repositories/contacts.rs`
   `find_suppliers_by_invoice_iban`, `iban_sql_key`, route dans `crates/kesh-api/src/lib.rs` au bloc Comptable+) :
   portée société des **deux** côtés (facture et contact), factures annulées exclues, contacts archivés ou non
   fournisseurs exclus, `creditor_iban` **et** `creditor_qr_iban`, ordre `MAX(invoice_date) DESC` puis `contacts.id`,
   `LIMIT 3` en SQL, 404 inconnu/autre société, vide sans QR ; l'IBAN n'est ni dans l'URL, ni dans la réponse, ni dans
   un journal (`tracing`). Injection SQL (le `format!` n'interpole que des noms de colonne constants ?), caractères
   U+00A0/U+202F dans le littéral SQL (encodage de la connexion), ambiguïté de colonnes dans le `JOIN`.
4. **L'écran** — `frontend/src/lib/features/supplier-invoices/{SupplierInvoiceForm.svelte, SupplierCreatePanel.svelte,
   supplier-invoice-form.ts, supplier-proposals.ts}`, `frontend/src/lib/shared/utils/iban.ts`, la page
   `routes/(app)/supplier-invoices/new/+page.svelte`, la liste `routes/(app)/supplier-invoices/+page.svelte`, la page
   d'import, `bank-accounts/+page.svelte` (fonction partagée, condition CH/LI ajoutée — régression possible pour un
   QR-IBAN saisi en minuscules ou avec des blancs exotiques ?). Avec QR : champs en lecture seule, **aucune** coordonnée
   dans le corps, pas de scan ; sans QR : routage QR-IBAN / IBAN par `qrIbanHasValidQrIid` ; sans document : scan 12.4.
   Propositions : **jamais présélectionnées** (« un appariement propose, il ne crée jamais ») ; *Retenir* ajoute le
   contact à la liste ; motif affiché. Panneau : aucun `<form>` imbriqué, `isSupplier: true`, adresse pré-remplie pour
   un QR de type `S` seulement (mappage `creditorLine1`/`creditorLine2` → rue/numéro : juste pour SPC 2.2 ?), sonde
   « contacts proches » (course entre deux réponses, minuterie détruite), erreur affichée sans perte, *Annuler* ne crée
   rien ; saisie intacte après création. Refus du serveur : `INVALID_PAYMENT_DETAILS` sous le champ, autres en tête,
   **saisie conservée**. Page `new` : document inconnu / autre société / plus à traiter / paramètre illisible → même
   message et lien ; état de chargement, état d'erreur ; rôle non Comptable+ : rien n'est proposé ni chargé (et le
   bouton de la liste masqué) ; après succès, navigation vers la fiche. Svelte 5 : `$bindable` de `suppliers`,
   `$derived` sur `document`, valeurs initiales du panneau lues une fois. Mise en page : aperçu fixe à gauche ≥ 1024 px,
   empilé en dessous, le document reste visible pendant la saisie ?
5. **Les propositions par nom** (`supplier-proposals.ts`) contre AC14 : ordre des étapes de `supplierNameKey` (NFC,
   trémas, `fold`, ponctuation, mots, formes juridiques de la plus longue à la plus courte, mots de liaison), clé
   triée sans doublon ; `searchTermsFor` (variantes à tréma et sans `e`, mots < 3 caractères omis) ; éligibilité
   (égalité ou inclusion, la plus petite non vide) **avant** classement ; IBAN avant nom, trois au plus, sans doublon,
   contacts inactifs ou non fournisseurs écartés. Cherche un nom réel qui ferait proposer à tort un fournisseur sans
   rapport, ou qui ferait disparaître le bon (« S.A. », « AG & Co. KG », « Sàrl » collé, chiffres, apostrophes,
   ligatures « œ »).
6. **i18n et gardes** : 52 clés neuves (50 `supplier-invoices-*`, `error-invalid-payment-details`,
   `error-qr-fields-from-document`) et 24 retirées (23 `imported-supplier-invoices-*`, `supplier-invoices-form-close`)
   **×4 locales**, parité, vraies traductions, mêmes variables, replis égaux à fr-CH au caractère près ; chaque clé
   retirée est-elle vraiment orpheline (`grep -rnF` dans `frontend/src` **et** `crates`) ? `imported-supplier-invoices-completed`
   gardée (glossaire). `sitesTotal` (2219 + 32 = 2251 après le report sur la 30-1b), `CLES_RELEVEES` 302 → 328,
   `boutons-rouges-inventory` 14 → 13 : **recomptés depuis la source**. `lint-i18n-ownership` : aucune violation
   ajoutée à la liste connue.
7. **Les tests prouvent-ils ce qu'ils nomment.** Rust : `inbox_import_e2e.rs` (cinq tests neufs de la section C) et
   `contacts.rs` (`search_terms_for_mueller_find_the_three_spellings`) ; Vitest : `iban.test.ts`,
   `supplier-proposals.test.ts`, `SupplierInvoiceForm.test.ts`, `new-page.test.ts`, `import-page.test.ts` ; E2E :
   `supplier-documents.spec.ts` (E1, E2, E4), `inbox-import.spec.ts` (création sur la page `new`), `helpers/inbox.ts`
   (helpers déplacés). La campagne (journal `kesh-gate-logs/302b-mutations.log`, motifs sous
   `kesh-gate-logs/302b-mut/*/`) : chaque « tuée » l'est-elle **pour la bonne raison** (`kesh-gate-logs/302b-mut-<id>.log`) ?
   Cherche la **mutation plausible qui resterait verte** (exemples : `present()` qui ne normalise plus ; `BOTH_IBANS`
   retiré ; `QR_IBAN_IN_IBAN_FIELD` sur un QR-IBAN en minuscules ; propositions qui ignorent `active` ; *Retenir* qui ne
   change pas la sélection ; le panneau qui oublie `isSupplier` ; la page qui ne masque pas le formulaire pour
   Consultation). Un test qui passerait à vide ?
8. ⛔ **LA DOCUMENTATION — manuels et PDF, `api-external.md`, CHANGELOG, README.** Chaque phrase neuve ou modifiée
   dit-elle vrai **au code** ? Manuel utilisateur (`docs/manual/fr/user-manual.tex`) : § *Saisir une facture
   fournisseur* (`sec:saisie-facture-fournisseur` — page de saisie, rôles, panneau, contrôle des coordonnées, note sur
   les pays absents et le lot pain.001) et § *Documents à traiter* (`sec:import-factures` — *Créer une facture*, côte à
   côte, champs verrouillés, propositions et leur avertissement, refus) ; K5. Manuel d'administration
   (`docs/manual/fr/admin-manual.tex`) : § `sec:inbox-import` et **§ *Passer à la 0.14.0*** (`sec:maj-0-14-0`,
   paragraphe *Intégrations par l'API* : `INVALID_PAYMENT_DETAILS`, corps de la complétion, `QR_FIELDS_FROM_DOCUMENT`,
   `supplier-candidates`). `docs/api-external.md` : section *Documents à traiter* (complétion, tableau des sept motifs,
   `supplier-candidates`) et tableau des codes d'erreur — vérifie chaque code, statut et champ contre `errors.rs` et
   les routes. CHANGELOG `## [0.14.0] — Non publié` : bandeau *À faire* (item *Intégrations*), *Ajouté*, *Modifié* —
   et la **réunion avec les entrées de la 30-1b** au report (rien de perdu, rien de dupliqué) ; `[0.13.4]` intact.
   README l. 44 (« import lancé depuis l'écran » gardé, garde G5 de `textes_coherents.rs`). Contrôle les **PDF
   aplatis** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` et de même `admin-manual.pdf`,
   **vers la sortie standard** — ⚠️ ligatures (« ﬀ », « ﬁ ») et apostrophe typographique (« ’ ») rendent un `grep`
   naïf faux négatif. Les PDF sont-ils postérieurs à leur `.tex` ? Grepe la **valeur** des textes que la 30-2a avait
   rendus vrais pour son état (« sans IBAN ni référence », « ni IBAN ni référence », « weder IBAN », « né IBAN », « no
   IBAN or reference », « Compléter », « complét ») sur `docs`, `README.md`, `website`, `crates` (doc-comments, quatre
   locales) et `frontend/src` : chaque site restant est-il juste ? Ce qui revient à la 30-2c n'y est **pas** annoncé ?

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Les chemins d'erreur de la complétion
  (refus après verrou, relecture post-`COMMIT`), la cohérence des deux routes, la sécurité de `supplier-candidates`
  (IDOR, fuite d'IBAN, coût d'un balayage), Svelte 5 (réactivité de `bind:suppliers`, `$derived`, panneau recréé),
  l'interaction formulaire ↔ panneau (le bouton *Créer le fournisseur* peut-il soumettre la facture ? la touche
  Entrée dans un champ du panneau ?), la navigation (`goto` après succès ; double clic sur *Enregistrer*).
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** IBAN de longueur limite, pays
  inconnu, minuscules, blancs exotiques (U+2007, U+3000, U+180E), QRR à 26/28 chiffres, référence de 64/65 caractères
  après `trim`, référence faite de blancs avec un QR-IBAN, `expectedPaymentAmount` à `0` ou négatif, document avec QR à
  montant ouvert, QR-IBAN dans un document à IBAN ordinaire, contact homonyme, plus de 200 fournisseurs, document
  traité dans un autre onglet pendant la saisie ; rejoue mentalement la campagne de mutations (axe 7).
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (AC10 à AC17, AC12-bis) contre le
  code : satisfait, partiel, absent ; T-b1 à T-b7 ; M-b1 à M-b13 présentes et conformes (M-b2 déclarée équivalente) ;
  les écarts à la fiche sont-ils **tous** déclarés au Dev Agent Record ; les chiffres de gates et d'E2E déclarés
  correspondent-ils aux journaux `kesh-gate-logs/302b-*.log` ; échecs E2E jugés fichier par fichier contre
  `docs/testing.md` § *Les échecs attendus* ; `sprint-status.yaml` : clé `30-2b-creation-cote-a-cote` en `review`,
  commentaire daté ; **recompte depuis la source** chaque décompte du Dev Agent Record (tests neufs par fichier aux deux
  bornes, clés, mutations, gates) et vérifie son **périmètre**.

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/302b-review-p1-<B|E|A>.md` (ta lettre), et rien d'autre nulle
part :

- findings numérotés `B1…` / `E1…` / `A1…`, chacun avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`,
  **preuve** (sortie `grep -nF` ou `sed -n` copiée pour toute affirmation de présence ou d'absence), correctif
  proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** (axes 1 à 8 et ceux de ta lentille) — un
  « 0 finding » sans cette liste ne compte pas comme passe ;
- un décompte final `CRITICAL/HIGH/MEDIUM/LOW`.
