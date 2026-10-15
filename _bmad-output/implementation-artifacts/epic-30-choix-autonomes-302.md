# Epic 30 — registre des choix autonomes de la Story 30-2 (à verser par union dans `epic-30-choix-autonomes.md`)

Fiche : `30-2-documents-factures-fournisseur.md`. Issue [#685]. Choix tranchés à la rédaction de la fiche, le
2026-10-15, en autonomie (Opus 5.5), sur `15686d55`. Format : contexte, retenu, écartées, réversibilité.

[#685]: https://github.com/guycorbaz/kesh/issues/685

---

### C-30-2-1 — Trois sous-stories, le socle d'abord

- **Contexte** : la story touche `kesh-db` (migration, entité, dépôt), `kesh-api` (import, routes des pièces, factures
  fournisseur, écritures, CSP, export CSV), `kesh-i18n`, le frontend (quatre écrans, contacts) et la documentation —
  plus de cinq modules : règle de découpage préventif du `CLAUDE.md`.
- **Retenu** : **30-2a** (tout document entre dans la liste, prévisualisation, retrait — la migration et le socle de
  la prévisualisation), **30-2b** (créer la facture à côté du document, fournisseur proposé ou créé sans quitter),
  **30-2c** (joindre après coup, consulter depuis la facture et l'écriture). Chacune livrable seule : après la 30-2a,
  un document sans QR se complète déjà par le formulaire existant. Une fiche, trois clés, trois PR successives ;
  `refs #685` sur les deux premières, `closes #685` sur la PR de la 30-2c seulement. Le découpage indicatif du
  fichier d'epic est confirmé, à un déplacement près : la prévisualisation va dans la 30-2a (elle sert la liste),
  la consultation depuis l'écriture dans la 30-2c.
- **Écartées** : une story unique (au-delà du seuil, et sept écrans à revoir d'une seule lentille) ; quatre
  sous-stories (le fournisseur en ligne séparé de la création côte à côte : la création n'a pas de sens livrée sans
  lui, la 30-2b tient en quatre modules).
- **Réversible** : oui, avant le développement.

### C-30-2-2 — Garder la table `imported_supplier_invoices`, rendre ses colonnes QR facultatives

- **Contexte** : la table de staging exige les colonnes du QR (`creditor_iban`, `is_qr_iban`,
  `creditor_address_type`, `creditor_name`, `creditor_country`, `reference_type`, `currency` : `NOT NULL`,
  `20260629000001_imported_supplier_invoices.sql`). Un document sans QR n'y entre pas.
- **Retenu** : une migration rend ces sept colonnes `NULL`, pose une contrainte « tout ou rien » (les sept nulles
  ensemble ou renseignées ensemble) et remplace l'index `idx_imported_supplier_invoice` par une contrainte d'unicité
  sur `supplier_invoice_id` (« une facture = un document », filet de la 30-2c). « Le document porte un QR » se lit
  `creditor_iban IS NOT NULL` ; aucune colonne neuve. Le nom de la table ne change pas (un `RENAME TABLE` serait
  breaking pour rien) ; l'écran dit « documents ».
- **Écartées** : (1) une table générique `documents` reliée aux factures — le bon modèle pour #235, mais une
  reprise des lignes existantes (backfill, triage P7, inventaire de sauvegarde, export CSV) hors de proportion ; (2)
  des valeurs sentinelles (`''`, `'CHF'`, `'CH'`, `'NON'`) sans migration — elles inventeraient une devise et un pays
  à un document qui n'en a pas, et `complete_import` routerait un IBAN vide (`Some("")`) : un faux muet.
- **Réversible** : difficilement après publication (une migration publiée ne se défait pas, P8).

### C-30-2-3 — La migration est breaking : `min_required` 0.14.0 et crates 0.14.0 dans la même PR

- **Contexte** : un binaire 0.13.x lit ces colonnes dans des `String` non optionnelles : une ligne sans QR fait
  échouer son décodage (liste, export CSV). `MODIFY COLUMN` entre dans la liste de P3.
- **Retenu** : dernière instruction de la migration `UPDATE _kesh_version SET kesh_version_min_required = '0.14.0'
  WHERE id = 1;` ; **les dix crates passent à 0.14.0 dans la même PR** (P2-bis) ; exemption au registre de rejeu
  (`EXEMPT_MIGRATIONS`, justification standard des bumps, précédents `20261009000001` et `20261011000001`) et
  compteur d'exemptions mis à jour. Si la 30-1 a déjà porté les crates à 0.14.0, la 30-2a le constate et ne bumpe
  que `min_required`.
- **Écartées** : `-- breaking-skip-bump` (P4) — la justification serait fausse : l'ancien binaire lit ces colonnes.
- **Réversible** : non après publication.

### C-30-2-4 — Un QR illisible n'empêche plus l'entrée du document

- **Contexte** : aujourd'hui, `INVALID_SPC_PAYLOAD`, `INVALID_IBAN`, `PDF_RENDER_ERROR` et `FIELD_TOO_LONG`
  envoient le fichier dans `failed/`, comme `NO_QR_CODE_FOUND`. Le parcours voulu fait entrer tout document PDF ou
  image ; la saisie à la main couvre un QR inexploitable.
- **Retenu** : ces cinq cas donnent un document **accepté sans données QR**, avec au rapport `qr: "none"` (aucun QR)
  ou `qr: "unreadable"` et le code d'origine en `qrError` *(précisé le 2026-10-15, validation P4, R2 : « le code d'origine
  du décodage, ou le code du contrôle pur » — `FIELD_TOO_LONG`, `QR_AMOUNT_OUT_OF_RANGE`)*. Le champ trop long se détecte **avant** l'insertion
  (longueurs des colonnes contrôlées sur la valeur à insérer) ; la branche `DataLengthOrRange` reste en filet. Restent
  refusés comme aujourd'hui : type hors liste blanche ou octets incohérents, image non décodable
  (`ImageDecode` → `UNSUPPORTED_FILE_TYPE`), taille, lien symbolique, doublon, erreur de lecture.
- **Écartées** : garder `failed/` pour le QR invalide (le document serait perdu pour la saisie, contre le parcours) ;
  accepter aussi l'image non décodable (le navigateur ne l'afficherait pas davantage).
  *(Corrigé le 2026-10-15, revue de code P1 de la 30-2a, B3 = E6 : la justification est fausse en partie. La crate
  `image` refuse des images que Chromium et Firefox affichent — JPEG CMYK ou à codage arithmétique, certaines variantes
  12 bits *(annoté le 2026-10-15, passe ciblée P2, T1 : le refus du JPEG CMYK n'est pas établi — `image` 0.25 et `zune-jpeg` le convertissent en RVB ; le manuel ne le cite plus)* — et toute image de plus de 10 000 pixels de côté (`MAX_IMAGE_DIMENSION`, `qr_decode.rs`) ; elles partent
  dans `failed/` sous `UNSUPPORTED_FILE_TYPE`, code qui ne dit pas pourquoi. Le refus est maintenu — il couvre le cas
  courant, l'image réellement corrompue ou tronquée —, mais comme **angle mort assumé**, écrit à la fiche, et non plus
  comme un cas sans perte.)*
- **Réversible** : oui.

### C-30-2-5 — « Supprimer de la liste » est l'actuel *Écarter*, sous un libellé neuf

- **Contexte** : Guy demande de « supprimer de la liste » un document qui n'est pas une facture ou un doublon.
  `discard_import` (`routes/imported_supplier_invoices.rs:336`) passe la ligne `discarded`, garde le fichier, et un
  nouveau dépôt du même fichier la réactive (`inbox_import.rs`, réactivation, Story 25-1b).
- **Retenu** : le geste reste `discard`, libellé « Retirer de la liste », avec confirmation ; fichier conservé,
  trace d'audit inchangée.
- **Écartées** : supprimer la ligne et le fichier — le stockage est adressé par contenu et peut être partagé (avec
  une autre société, ou un PDF figé de facture au même octet près : commentaire `inbox_import.rs:556-567`) ; une
  suppression à tort ferait répondre 410 à une pièce émise.
- **Réversible** : oui.

### C-30-2-6 — Prévisualiser par un blob, et élargir la CSP à `blob:`

- **Contexte** : la CSP des pages (`middleware/csp.rs:24-32`) est `default-src 'self'; img-src 'self' data:` — un
  `<img>` ou un `<iframe>` sur une URL `blob:` y est bloqué. Le téléchargement actuel passe par
  `apiClient.getBlob`, qui gère le rafraîchissement du jeton (cookie httpOnly) ; une `<iframe>` pointée sur l'URL de
  l'API ne le gère pas et afficherait un 401 brut à l'expiration du jeton.
- **Retenu** : le composant de prévisualisation lit le document par `apiClient.getBlob`, en fait une URL `blob:`
  (révoquée à la destruction), l'affiche en `<img>` (PNG, JPEG) ou en `<iframe>` (PDF), et pour tout autre type un
  simple lien de téléchargement ; la CSP gagne `blob:` dans `img-src` et deux directives **explicites**,
  `frame-src 'self' blob:` et `object-src 'self' blob:` (validation P1, F5/R6 : `object-src` ne reste pas hérité de
  `default-src`). Aucun changement du `Content-Disposition` de l'API. **Amendé le 2026-10-15 (validation P1)** : aucun
  E2E ne voit un PDF s'afficher (Chromium sans tête n'a pas de visionneuse) ; un relevé dans un vrai navigateur à
  fenêtre est exigé au Dev Agent Record (AC6-bis), et la CSP se corrige s'il échoue.
- **Écartées** : `<iframe src="/api/v1/…/source-document">` avec un `Content-Disposition: inline` (401 brut à
  l'expiration, pas d'état d'erreur maîtrisé) ; un rendu pdf.js embarqué (dépendance lourde pour un aperçu).
- **Réversible** : oui.

### C-30-2-7 — Le fournisseur est proposé, jamais présélectionné

- **Contexte** : règle « un appariement propose, il ne crée jamais » ; le faux rattachement est le danger muet.
- **Retenu** : à partir du nom du créancier du QR, jusqu'à trois fournisseurs actifs proposés, chacun avec un bouton
  « Retenir » ; la liste déroulante reste vide tant que l'utilisateur n'a pas retenu ou choisi. Recherche et
  classement réutilisent le socle des contacts (`listContacts({ search, isSupplier: true })` puis
  `normalizeTerm`/`rank`, `features/contacts/duplicate-probe.ts`). Sans QR, aucune proposition. **Amendé le 2026-10-15
  (validation P1, R1)** : la recherche serveur est un « OU » de mots et `rank` ne filtre pas ; une fonction pure
  `proposeSuppliers` filtre d'abord (tous les mots du plus court des deux noms, formes juridiques et mots de liaison
  retirés, figurent dans l'autre), puis classe. Un fournisseur qui ne partage que « Sàrl » n'est plus proposé.
  **Amendé de nouveau le 2026-10-15 (validation P2, R7, arbitrage de l'orchestrateur : le rappel avant la précision)** :
  deux sources dans l'ordre — les fournisseurs dont une facture passée porte l'IBAN ou le QR-IBAN du créancier (lecture
  neuve `GET /imported-supplier-invoices/{id}/supplier-candidates`, l'IBAN lu sur le document), puis le nom par une clé
  normalisée (`supplierNameKey` : trémas développés `ae`/`oe`/`ue`, accents repliés, ponctuation en espaces, formes
  juridiques retirées comme suites de mots, mots de liaison retirés) avec égalité ou inclusion d'ensembles de mots ;
  trois au plus, motif affiché, aucun présélectionné. La règle « tous les mots du plus court » de la P1 n'était pas
  décidable (tokenisation, mesure) et écartait « Dubarde Sàrl Genève » et « Mueller AG ».
- **Écartées** : présélectionner le premier (un enregistrement sans regarder rattacherait au mauvais fournisseur) ;
  un appariement par IBAN (les contacts ne portent pas d'IBAN) *(écartée levée le 2026-10-15 par l'amendement P2 ci-dessus :
  l'IBAN se lit sur les factures passées, non sur les contacts)*.
- **Réversible** : oui.

### C-30-2-8 — Le fournisseur se crée dans un panneau de la page, au minimum

- **Contexte** : « le créer sans quitter la création de la facture, puis continuer ». Le formulaire de contact de
  `routes/(app)/contacts/+page.svelte` (1161 lignes) n'est pas un composant.
- **Retenu** : un panneau de la page de création : type (Entreprise par défaut), nom (pré-rempli du créancier QR),
  adresse structurée pré-remplie quand le QR est de type `S` (rue, numéro, NPA, localité, pays), `isSupplier` posé ;
  le signal « contacts proches » du socle 22-2a s'affiche avant l'enregistrement (signaler, jamais bloquer). À
  l'enregistrement, le contact créé est ajouté à la liste et **sélectionné** — c'est le geste humain qui le désigne.
  La saisie de la facture (lignes comprises) n'est pas perdue ; *Annuler* ferme le panneau sans rien créer.
- **Écartées** : extraire le formulaire complet des contacts (refonte d'un écran hors périmètre) ; ouvrir la page des
  contacts dans un onglet (on quitte la saisie) ; pré-remplir l'adresse d'un QR de type `K` (lignes combinées dont le
  découpage serait deviné).
- **Réversible** : oui.

### C-30-2-9 — Avec QR, les coordonnées de paiement viennent du document et ne se modifient pas

- **Contexte** : `complete_import` prend aujourd'hui IBAN, référence et montant dans le staging et ignore le corps.
- **Retenu** : à l'écran, IBAN ou QR-IBAN, référence et montant attendu sont affichés pré-remplis et en lecture
  seule ; côté serveur, un corps qui porte l'un de ces champs pour un document **avec** QR est refusé en `400`
  `QR_FIELDS_FROM_DOCUMENT` (un écart silencieux entre ce qu'on a saisi et ce qui est enregistré serait muet).
  Sans QR, ils viennent du corps, validés comme `POST /supplier-invoices`.
- **Écartées** : laisser modifier (le document dit autre chose que la facture) ; ignorer silencieusement le corps
  (comportement actuel, acceptable tant que l'écran ne montrait pas ces champs).
- **Réversible** : oui.

### C-30-2-10 — La complétion reçoit le corps de `POST /supplier-invoices`

- **Contexte** : `CompleteImportRequest` (`routes/imported_supplier_invoices.rs`) double
  `CreateSupplierInvoiceRequest` (`routes/supplier_invoices.rs:238`) moins les champs de paiement et le projet
  (« extension future → None », l. 269).
- **Retenu** : `complete_import` désérialise `CreateSupplierInvoiceRequest` ; la construction de `NewSupplierInvoice`
  est extraite en une fonction commune aux deux routes ; `project_id` est honoré. Compatible avec les clients
  actuels (champs ajoutés optionnels). Cette fonction porte aussi la validation des coordonnées de paiement
  (C-30-2-20).
- **Écartées** : garder deux types (règle DRY).
- **Réversible** : oui.

### C-30-2-11 — Une page de création, un composant de formulaire

- **Contexte** : le formulaire de saisie vit en ligne dans `routes/(app)/supplier-invoices/+page.svelte`, et un
  second, plus pauvre, dans `routes/(app)/supplier-invoices/import/+page.svelte` (complétion).
- **Retenu** : `SupplierInvoiceForm.svelte` (dans `features/supplier-invoices/`) extrait du premier ; page
  `/supplier-invoices/new` qui l'affiche seul (« sans document », avec le scan navigateur 12.4) ou à côté du document
  (`?document=<id>`, aperçu fixe à gauche, formulaire à droite ; empilés sous 1024 px). Le bouton de la liste mène à
  `/supplier-invoices/new` ; *Compléter* devient « Créer une facture » et mène à `?document=<id>` ; le formulaire de
  complétion en ligne disparaît.
- **Écartées** : garder les deux formulaires (DRY) ; un panneau modal (le document doit rester visible pendant toute
  la saisie et la répartition des montants).
- **Réversible** : oui.

### C-30-2-12 — Joindre après coup depuis la fiche de la facture, sans contrôle de montant

- **Contexte** : point 7 du parcours ; l'entrée reste le dossier, donc le document à joindre est un document à
  traiter.
- **Retenu** : « Joindre un document » sur la fiche d'une facture fournisseur **sans** document, quel que soit son
  statut (ouverte, payée, annulée) : sélecteur des documents à traiter, avec aperçu, montant QR affiché à côté du total
  de la facture ; route `POST /supplier-invoices/{id}/source-document` `{ importedSupplierInvoiceId }`. Aucun champ de
  la facture n'est modifié, aucun refus sur un écart de montant : l'humain voit les deux et décide.
- **Écartées** : refuser un écart de montant (une facture peut avoir été saisie à un montant corrigé) ; joindre
  depuis la liste des documents (deux entrées pour un même geste ; l'utilisateur part de la facture qui lui manque).
- **Réversible** : oui.

### C-30-2-13 — Le détachement entre dans la 30-2c, au plus simple (décision de l'orchestrateur, 2026-10-15)

- **Contexte** : un document joint par erreur ne se détachait pas. La fiche proposait d'en faire une limite avec une
  issue ; l'orchestrateur l'a refusé : une dette sans issue est interdite (rétrospective 28), et ouvrir l'issue
  casserait la règle de release (marge d'une issue au tag v0.14.0).
- **Retenu** : AC24 — « Détacher le document » sur la fiche de la facture (Comptable+, confirmation),
  `DELETE /supplier-invoices/{id}/source-document` : le document repasse `to_complete` avec `supplier_invoice_id =
  NULL` et revient dans « Documents à traiter » ; la facture, ses lignes et ses écritures sont inchangées ; audit
  `imported_supplier_invoice.detached` ; tout statut de facture. La transaction ne verrouille que la ligne du document
  (pas de cycle avec le rattachement, AC19). Vérifié au code : aucune contrainte ne lie `status` et
  `supplier_invoice_id`, la réactivation écrit déjà ce `NULL`, l'unicité de l'AC4 admet plusieurs `NULL`.
- **Écartées** : limite écrite avec issue (refusée, ci-dessus) ; supprimer le document au détachement (contre
  C-30-2-5) ; refuser le détachement d'une facture payée (le lien n'a aucun effet comptable).
- **Angle mort** : un document avec QR détaché peut servir à créer une seconde facture ; geste humain, dit au manuel.
- **Réversible** : oui.

### C-30-2-14 — La lecture du document d'une facture s'ouvre à tout rôle authentifié *(titre corrigé le 2026-10-15, validation P2, L1 : il disait « les deux lectures », contre son propre Retenu)*

- **Contexte** : `GET /imported-supplier-invoices/{id}/source-document` et `GET /supplier-invoices/{id}/source-document`
  sont montées parmi les routes Comptable+ (`lib.rs:506-513`), alors que la facture et l'écriture se lisent par tout
  rôle authentifié (`lib.rs:868-875`). « Consultable depuis la facture et depuis son écriture » échouerait en 403 pour
  le rôle Consultation.
- **Retenu** : `GET /supplier-invoices/{id}/source-document` rejoint les lectures (tout rôle authentifié, toujours
  bornée à la société) ; la lecture d'un document **à traiter** reste Comptable+ (la liste l'est).
- **Écartées** : masquer les boutons pour le rôle Consultation (la pièce resterait invisible à qui lit les livres).
- **Réversible** : oui.

### C-30-2-15 — L'écriture d'achat et l'écriture de règlement montrent le document

- **Contexte** : une facture fournisseur possède deux écritures (`purchase_journal_entry_id`,
  `settlement_journal_entry_id`, `DocumentKind::SupplierInvoice`, `journal_entries.rs:2073-2087`).
- **Retenu** : le détail d'une écriture expose `sourceDocument` (`supplierInvoiceId`, numéro de facture fournisseur,
  nom du fichier) si l'écriture est l'achat **ou** le règlement d'une facture fournisseur qui a un document ; la page
  affiche le lien vers la facture et « Voir le document ».
- **Écartées** : l'achat seul (le règlement par la réconciliation de la 30-1 est l'écriture que Guy ouvre depuis le
  relevé).
- **Réversible** : oui.

### C-30-2-16 — #503 reste hors périmètre, signalé comme préalable à la mise en production

- **Contexte** : `.keshbackup` n'emporte aucun fichier (`admin_backup/export.rs:94-97`, `files/` vide ;
  `admin_backup/import.rs:91` refuse un `files/` non vide). La 30-2 fera entrer **davantage** de pièces dans
  `KESH_DOCUMENTS_DIR`.
- **Retenu** : aucune modification de la sauvegarde ; le manuel utilisateur, à la section des documents, renvoie à la
  sauvegarde de `KESH_DOCUMENTS_DIR` décrite au manuel administrateur (l. 793, 1456, 1599) ; le rapport demande à
  l'orchestrateur de porter #503 à l'attention de Guy avant la mise en production.
- **Écartées** : intégrer #503 (changement du format de sauvegarde, version du manifeste, vérification d'empreintes :
  une story à elle seule, non mécanique).
- **Réversible** : oui.

### C-30-2-17 — Le dossier se vide à l'import, comme aujourd'hui

- **Contexte** : #685 dit qu'« après création » le document « quitte la liste à traiter et le dossier d'entrée ». Au
  code, le fichier quitte le dossier **à l'import** : archivé sous `KESH_DOCUMENTS_DIR`, supprimé de l'inbox
  (`inbox_import.rs:623`) ; seul un échec le laisse, dans `failed/`.
- **Retenu** : comportement conservé ; la création fait quitter la **liste** (statut `completed`). Prémisse corrigée
  dans la fiche, à commenter sur l'issue.
- **Écartées** : laisser le fichier dans le dossier jusqu'à la création (chaque import le relirait et le
  signalerait en doublon).
- **Réversible** : oui.

### C-30-2-18 — « Documents à traiter », adresse de page inchangée

- **Contexte** : « Importer des factures », « Factures à compléter », « Compléter » disent une facture ; la liste
  contiendra des documents qui n'en sont pas.
- **Retenu** : titre et menu « Documents à traiter », section « Documents à traiter », bouton « Créer une facture »,
  contrôle K5 de la préparation de clôture « Documents fournisseur à traiter » ; l'URL `/supplier-invoices/import`
  reste (liens de `FiscalYearChecks.svelte`). Les clés i18n gardent leur nom, seuls les textes changent dans les quatre
  locales ; la clé `imported-supplier-invoices-error-no-qr-code-found`, qui n'a plus d'émetteur, est retirée avec son
  entrée de `error-label.ts`.
- **Écartées** : renommer la route (liens et E2E à reprendre pour rien).
- **Réversible** : oui.

### C-30-2-19 — Un document sans QR n'a pas de devise : la facture est en CHF

- **Contexte** : `complete_import` refuse une devise autre que CHF (`routes/imported_supplier_invoices.rs:183`), lue
  dans le staging.
- **Retenu** : le contrôle ne s'applique qu'à un document avec QR ; sans QR, la facture est en CHF comme toute
  facture fournisseur saisie.
- **Écartées** : demander une devise (les factures fournisseur sont en CHF seulement).
- **Réversible** : oui.

### C-30-2-20 — Les coordonnées de paiement se valident dans la conversion commune, pour les deux routes

- **Contexte** : validation P1 (F6, R14). `POST /supplier-invoices` ne valide ni l'IBAN, ni l'exclusivité IBAN /
  QR-IBAN, ni la cohérence QRR : `create_in_tx` insère tel quel (`repositories/supplier_invoices.rs:340-343`,
  `:526-539`). Arbitrage de l'orchestrateur : prescrire la validation.
- **Retenu** (réécrit le 2026-10-15 après la validation P2 — F1-3, F1-4, R6, arbitrage de l'orchestrateur ; la première
  rédaction s'appuyait sur `kesh_qrbill::validation::validate_iban`, qui n'accepte que CH/LI à 21 caractères et aurait
  refusé tout IBAN étranger) : AC12-bis — un seul type par côté. Serveur : `kesh_core::types::Iban` (international,
  MOD-97) pour `creditorIban`, qui ne doit pas être un QR-IBAN ; `kesh_core::types::QrIban` (CH/LI, QR-IID 30000-31999)
  pour `creditorQrIban` ; pas les deux. QR-IBAN ⇒ référence QRR obligatoire et valide (`validate_qrr`) ; IBAN ordinaire
  ou absent ⇒ référence en texte libre (≤ 64), **27 chiffres compris** *(amendé le 2026-10-15, validation P3, F3-3 : la
  P2 refusait une « forme QRR » sans QR-IBAN, ce qui rejetait l'ancienne référence BVR/ESR d'une facture ancienne sans
  bénéfice en aval)*. Normalisation par **une** fonction par côté, qui retire tous les blancs Unicode (insécables
  compris) des IBAN *(amendé en P3, R3 = F3-7 : `Iban::new` n'ôte que l'espace ASCII ; précisé le 2026-10-15, P4 F4 et P5
  T5 : l'ensemble retiré est `White_Space` d'Unicode plus U+200B et U+FEFF, la même liste des deux côtés — AC12-bis)* ; chaîne vide ou blanche =
  absente ; valeurs stockées normalisées. Sept motifs de refus, chacun avec son libellé dans les quatre locales (P3, R4).
  L'effet sur la création manuelle se décrit exactement : table des pays et longueur par pays d'`Iban::new` (des pays
  absents, refusés), lot pain.001 limité à CH/LI (existant). Refus `400 INVALID_PAYMENT_DETAILS`
  (`details.field`, `details.reason`). Frontend : la fonction existante `qrIbanHasValidQrIid`
  (`bank-accounts/+page.svelte:276`) extraite et partagée, complétée de la condition CH/LI. Effet sur la création
  manuelle écrit tel quel au CHANGELOG : refus d'un IBAN faux, d'un QR-IBAN mal placé, des deux ensemble, d'un QR-IBAN
  sans QRR valide ; un IBAN étranger valide (pays de la table) et une référence libre restent acceptés.
- **Écartées** : `kesh_qrbill::validation::validate_iban` (CH/LI seulement : régression silencieuse pour les
  fournisseurs étrangers) ; « aucun contrôle, comme aujourd'hui » en angle mort (dette sans issue interdite) ; valider
  la seule complétion (deux chemins pour une même saisie).
- **Réversible** : oui.

### C-30-2-21 — Le rattachement ne lit pas le lien existant : l'unicité refuse

- **Contexte** : validation P1 (F1, R10). L'étape 3 prévue (lecture verrouillante d'un document déjà lié, sur l'index
  d'unicité) pose un verrou d'intervalle quand aucune ligne n'existe — le cas nominal — et fait s'interbloquer deux
  rattachements vers deux factures voisines ; le rejeu le masquerait. Arbitrage de l'orchestrateur : la voie la plus
  simple.
- **Retenu** : plus de lecture préalable ; après les verrous du staging et de la facture (clés primaires), l'`UPDATE`
  de `mark_completed` rencontre l'unicité de `supplier_invoice_id`, dont la violation se traduit en
  `409 SOURCE_DOCUMENT_ALREADY_ATTACHED`. La transaction ne fait aucune lecture ordinaire (aucun instantané n'y naît).
  Tests sans rejeu (fonctions `_once`) : deux factures différentes, deux succès ; même facture, un `409`.
- **Écartées** : lecture ordinaire après les deux verrous (correcte aussi, mais un second chemin pour le même refus et
  une mutation équivalente de plus) ; garder la lecture verrouillante (verrou d'intervalle).
- **Réversible** : oui.
- **Amendé le 2026-10-15 (validation P3, R2 = F3-1, relevé de l'orchestrateur)** : les deux validations soupçonnaient
  le contrôle de doublon d'InnoDB de poser seul un verrou sur le successeur. **Mesuré au banc** (MariaDB 10.11.16 de dev,
  `REPEATABLE-READ`, tables jetables `inv` / `doc` à `inv_id NULL UNIQUE` et clé étrangère ; A verrouille doc 1 puis inv X
  et écrit le lien sans valider, B fait de même sur doc 2 et inv Y ; couples (10, 20), (20, 10), (30, 10), (10, 30), avec
  et sans liens existants 15 et 25) : B aboutit chaque fois en moins de 0,35 s. Test « B sous 5 s » gardé. **Bascule
  écrite** si le test rougit au développement : sérialisation par la sentinelle `companies … FOR UPDATE` en tête de
  transaction (patron de `supplier_invoices::create_in_tx`), test réécrit sur l'ordre d'événements. *(Corrigé le
  2026-10-15, validation P4, F1 = R1 : `create_in_tx` ne prend la sentinelle que pour une facture à projet, et une
  sentinelle en tête formait des cycles avec la complétion et le règlement ; la bascule prend
  `bank_accounts::acquire_company_sentinel_lock` **après** le staging et la facture, avant l'`UPDATE` — ordre staging →
  facture → `companies`, tableau des chemins à l'AC19.)*

### C-30-2-22 — Le contrôle pur couvre longueur, plage et domaine, avant l'archivage ; filet des contraintes CHECK *(titre à lire « longueur et plage » depuis le 2026-10-15, validation P3 : le domaine est retiré ; le contexte ci-dessous, « 500 sur tout le run … défaut d'origine », était faux à sa date — voir le Retenu)*

- **Contexte** : validation P2 (F1-1, R1, R2, R3). Le montant du QR n'est borné par personne (`parser.rs:145`) ; le type
  d'adresse est pris sans contrôle (`parser.rs:120`) et une valeur hors `K`/`S` viole `chk_imported_si_address_type`,
  que l'import ne capte pas : 500 sur tout le run, fichier resté dans l'inbox, chaque import suivant en échec (défaut
  d'origine) ; le contrôle de la P1 était décrit comme une fonction de `NewImportedSupplierInvoice`, construite **après**
  l'archivage, tout en se disant « avant l'archivage ».
- **Retenu** : une fonction pure de `ScannedQrBill`, appelée avant l'archivage, vérifie longueur (en caractères) et plage
  du montant (`amount_min`..`amount_max`, deux décimales significatives) ; *(le contrôle de domaine de
  `creditor_address_type` d'abord retenu est **retiré le 2026-10-15**, validation P3, R1 HIGH = F3-2 : `from_scanned`
  normalise déjà tout type ≠ `S` en `K`, `imported_supplier_invoice.rs:117-128`, test l. 256 — le « défaut d'origine »
  n'existait pas, et un `CheckConstraintViolation` rend 400, non 500 ; arbitrage P2 de l'orchestrateur pris pour acquis
  sans vérification, reconnu par lui)* ; un QR qui échoue entre
  sans ses données (`unreadable`). Un filet `CheckConstraintViolation` rejoint `DataLengthOrRange` à l'`INSERT`
  (`failed[]`) — défense en profondeur, sans test, aucune contrainte CHECK n'étant atteignable.
  *(Précisé le 2026-10-15, validation P4, R2 = F5 : un code `qrError` neuf, `QR_AMOUNT_OUT_OF_RANGE`, un seul pour la
  plage et les décimales, libellé ×4 ; le contrôle ne porte que sur un montant présent ; tout document `unreadable` dit
  à l'écran qu'il est accepté sans les données du QR.)* Le cas « nom de fichier trop long » est retiré : inatteignable (`NAME_MAX` = 255 octets).
- **Écartées** : laisser le filet seul décider (un QR exploitable en partie perdrait le document) ; garder un test de
  nom trop long (impossible à monter).
- **Réversible** : oui.

### C-30-2-23 — La date de règlement s'affiche formatée (décision de l'orchestrateur, 2026-10-15)

- **Contexte** : constat de la 30-1 — la fiche d'une facture fournisseur payée affiche `paidAt` brut
  (« 2026-06-20T00:00:00 », `supplier-invoices/[id]/+page.svelte:393`) ; avec la 30-1, `paid_at` devient la date du
  paiement à minuit.
- **Retenu** : AC25, dans la 30-2c — `formatSwissDate` existant (`reports.api.ts:193`) sur les dix premiers caractères,
  libellé adapté dans les quatre locales, Vitest, mutation M-c12.
- **Écartées** : une issue séparée (la marge d'issues au tag est d'une seule) ; un formateur neuf (DRY).
- **Réversible** : oui.

---

*Choix du développement de la 30-2a (2026-10-15, Opus 5.5, autonomie), sur `5b566378`, branche `story/30-2`.*

### C-30-2-24 — Le téléchargement reste à côté de l'aperçu

- **Contexte** : AC5 énumère trois gestes par ligne (Aperçu, Compléter, Retirer de la liste) ; la page portait aussi
  « Voir le justificatif », qui **télécharge** le document, et `inbox-import.spec.ts` l'exerce. Un PDF s'affiche dans la
  visionneuse du navigateur, qui sait l'enregistrer ; une image, non.
- **Retenu** : le bouton reste, libellé « Télécharger » (clé `imported-supplier-invoices-view-doc`, texte changé dans les
  quatre locales — le mot « justificatif » sort du vocabulaire, AC8). `DocumentPreview` garde son lien « Télécharger »
  pour les seuls types qu'il n'affiche pas (AC6).
- **Écartées** : retirer le bouton (une image ne s'enregistrerait plus qu'au clic droit, et la couverture E2E du
  téléchargement disparaîtrait) ; un lien de téléchargement dans tout aperçu (AC6 le réserve aux autres types).
- **Réversible** : oui.

### C-30-2-25 — Le contrôle pur du QR vit dans l'entité, et rend un type, non une chaîne

- **Contexte** : AC2 prescrit une fonction pure de `ScannedQrBill`, appelée avant l'archivage ; les largeurs de colonnes
  sont un savoir de `kesh-db`, les codes `qrError` un savoir du rapport d'import.
- **Retenu** : `check_scanned_fits_columns(&ScannedQrBill) -> Result<(), QrColumnsMismatch>` dans
  `kesh-db/src/entities/imported_supplier_invoice.rs`, à côté de `from_scanned` et de `without_qr` ;
  `QrColumnsMismatch::{FieldTooLong { column }, AmountOutOfRange}` et sa méthode `code()` (`FIELD_TOO_LONG`,
  `QR_AMOUNT_OUT_OF_RANGE`). Les deux filets de l'`INSERT` gardent les codes prescrits (`DataLengthOrRange` →
  `FIELD_TOO_LONG`, `CheckConstraintViolation` → `INVALID_SPC_PAYLOAD`, AC2).
- **Écartées** : la fonction dans `inbox_import.rs` (les largeurs y seraient recopiées loin du schéma) ; un `&'static str`
  nu (la colonne en défaut se perdrait, et un test ne pourrait pas la nommer).
- **Réversible** : oui.

### C-30-2-26 — Le vocabulaire du journal d'activité suit l'écran ; les messages d'erreur du serveur, non

- **Contexte** : AC8 fait inventorier les jetons « importée », « complét », « écart » dans les quatre locales ; les
  libellés d'audit (`audit-log-action-imported-supplier-invoice-*`, `audit-log-entity-*`) disaient « Facture importée
  créée / complétée / écartée / réactivée ». Les messages d'`AppError` du serveur (`errors.rs`, « Facture importée
  introuvable ») ne sont pas affichés par la page, qui traduit le code ; et la 30-1a modifie `errors.rs` en parallèle.
- **Retenu** : les cinq libellés d'audit passent au vocabulaire du document (« Document importé », « Document traité
  (facture créée) », « Document retiré de la liste », « Document remis dans la liste », entité « Document fournisseur »),
  dans les quatre locales. `errors.rs` n'est pas touché ; le message de `CURRENCY_NOT_SUPPORTED`, construit dans la route,
  devient « … Retirez le document de la liste. » (AC7).
- **Écartées** : garder « Facture importée » au journal (le journal nommerait « facture » un document qui n'en est pas
  une) ; réécrire les messages de `errors.rs` (conflit avec la 30-1a, sans effet à l'écran).
- **Réversible** : oui.

### C-30-2-27 — Le développement part de `story/30-2`, avant le merge de la 30-1a

- **Contexte** : T-a0 et C-30-0-3 font partir la 30-2a d'`origin/main` **après** le merge de la 30-1 ; l'orchestrateur a
  lancé le développement en parallèle de la 30-1a, et demandera un rebase après son merge.
- **Retenu** : développement sur `story/30-2` (`5b566378`) ; T-a0 reste à faire au rebase. La 30-1a n'ajoute aucune
  migration : l'horodatage `20261015000001` reste le plus récent, et les dix crates n'étaient pas à 0.14.0 — la 30-2a les
  y porte. Conflits probables écrits au Dev Agent Record.
- **Écartées** : attendre le merge (consigne de l'orchestrateur).
- **Réversible** : oui (rebase).

### C-30-2-28 — Les dossiers E2E du worktree sont nommés par la clé, et passés au runner

- **Contexte** : le spec lit `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR` avec pour défauts ceux de la recette
  (`/tmp/kesh-e2e/…`) ; d'autres agents peuvent monter la même recette au même moment, dans les mêmes dossiers.
- **Retenu** : le montage du worktree emploie `/tmp/kesh-e2e-302a/{inbox,documents,backup}`, passés au backend **et** au
  runner (consigne « fichiers temporaires nommés par ta clé ») ; les défauts du spec restent ceux de la recette.
- **Écartées** : les dossiers par défaut (un import d'un autre agent prendrait mes fichiers, et inversement).
- **Réversible** : oui.

### C-30-2-29 — Ce qui revient à la 30-2b et à la 30-2c n'est pas écrit d'avance

- **Contexte** : AC9-bis demande, au *Passer à la 0.14.0* et au bandeau du CHANGELOG, la validation des coordonnées de
  paiement (`INVALID_PAYMENT_DETAILS`, « à compléter par la 30-2b ») et la lecture du document ouverte aux clés `read`
  (30-2c) ; rien de cela n'est livré par la 30-2a.
- **Retenu** : la 30-2a n'écrit que ce qu'elle livre (champs nullables, `qr`/`qrError`, disparition de
  `NO_QR_CODE_FOUND`) ; les deux points s'ajouteront avec leur code. Écart à la fiche déclaré au Dev Agent Record.
- **Écartées** : annoncer au manuel un comportement absent du code (le manuel dirait faux jusqu'au merge de la 30-2b).
- **Réversible** : oui.
