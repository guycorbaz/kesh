# Release 0.17.2 — registre des choix autonomes

Registre de la release 0.17.2, **mis à jour par union** (`docs/agents-orchestration.md` § 1) : chaque story y tient sa
section, numérotée par sa clé ; une section d'une autre story ne se réécrit pas. Chaque choix : contexte, option retenue,
alternatives écartées, réversibilité.

---

## Story v0172-correctifs (clé `v0172b`) — #876, #840, #870, #851, #882, #880, #883, #885

Fiche : `v0172-correctifs.md`. Numéros de ligne de `d7ec2ea2`. Issues en `refs` seulement.

### C-v0172b-1 — Découpage en quatre sous-stories dans la même fiche *(révisé à la validation P1, 2026-11-04 : trois → quatre)*

- **Contexte** : huit issues, plus de cinq modules (règle de splitting préventif du `CLAUDE.md`).
- **Retenu** : quatre sous-stories posées dans `v0172-correctifs.md` — A1 (session #876), A2 (import simultané #840), B
  (écrans : #870, #851, #882, #885), C (forme : #880, #883) —, développées dans l'ordre A1 → A2 → B → C sur la branche
  `story/v0172-correctifs`, chacune close par ses commits, son gate et sa revue de code. Chacune touche au plus cinq
  modules (tableau complet des fichiers dans la fiche) ; C est un déploiement mécanique. #876 et #840, sans lien, ont été
  séparées à la validation P1 (finding P5 : réunies, elles dépassaient cinq modules). Une seule clé `v0172-correctifs` au registre des sprints, comme le mandat le
  demande.
- **Écartées** : (a) une story unique — trop large pour une revue adversariale fiable ; (b) trois fiches sœurs — le mandat
  permet la même fiche, et les trois partagent la contrainte, le CHANGELOG et le manuel ; (c) huit stories — coût de
  boucles de revue disproportionné pour des correctifs de P3/P4.
- **Réversible** : oui — une sous-story peut être sortie en fiche sœur `v0172-correctifs-<n>` si sa revue ne converge pas.

### C-v0172b-2 — #876 : renouvellement côté navigateur, à l'entrée de l'utilisateur, et champ `sessionExpiresIn` *(révisé à la validation P1)*

- **Contexte** : avec 15/15 minutes (défauts), le renouvellement sur 401 arrive toujours après l'expiration du jeton de
  renouvellement (`routes/auth.rs:254, :399` ; cookie d'accès `max_age = jwt_expiry`, `:54`) : la session meurt à 15 minutes,
  en pleine saisie, avec redirection vers `/login` (`api-client.ts:176-177`).
- **Retenu** : le frontend renouvelle **à l'entrée de l'utilisateur**, au plus une fois par pas `T` (C-v0172b-23), tant que la
  session vit ; le serveur rend `sessionExpiresIn` (login, refresh, changement de mot
  de passe, assistant : `refresh_inactivity` ; `/me` : durée restante du jeton du cookie, `null` sinon).
- **Écartées** : (a) relâcher l'expiration au serveur (`expires_at = now + jwt + leeway + inactivity`) — inactivité tolérée
  floue (15 à 31 minutes), saisie longue sans requête perdue quand même ; (b) prolonger à chaque requête — les sondes de fond
  (`BackupScheduleBanner`, `api-health`) rendraient la session éternelle ; (c) ne corriger que le manuel — la perte de saisie
  est le défaut ; (d) déduire l'échéance du renouvellement côté navigateur sans aide du serveur — impossible après un
  rechargement de page, et faux si `KESH_REFRESH_INACTIVITY_MINUTES < KESH_JWT_EXPIRY_MINUTES` ; (e) une minuterie armée avant
  l'échéance qui renouvelle s'il y a eu activité depuis le dernier renouvellement — la première rédaction : borne d'inactivité
  jusqu'à `2N − 1`, et minuterie muette pendant une veille ou ralentie dans un onglet masqué (findings P1-P2 de la validation P1).
- **Réversible** : oui (champ JSON additif, module frontend isolé).

### C-v0172b-3 — #876 : l'activité, ce sont les entrées de l'utilisateur, non les requêtes

- **Contexte** : des composants interrogent le serveur sans geste de l'utilisateur (`BackupScheduleBanner.svelte:34`,
  `api-health.svelte.ts:102`).
- **Retenu** : `keydown`, `pointerdown`, `wheel`, `touchstart` (écouteurs passifs sur `window`) et `visibilitychange` vers
  `visible` (revenir sur l'onglet est un geste) ; posés à l'armement par `authState`, retirés au désarmement (`clearSession`,
  `logout`) ; une saisie longue sans
  enregistrer garde la session ; un onglet laissé ouvert expire après l'inactivité réglée.
- **Écartées** : compter les requêtes (session éternelle sur un écran d'administration) ; compter `mousemove` (un pointeur
  qui frôle l'écran prolongerait la session d'un poste quitté) ; `scroll` (déclenché aussi par le code).
- **Réversible** : oui.

### C-v0172b-4 — #876 : un seul renouvellement pour tous les onglets

- **Contexte** : les onglets partagent les cookies ; un jeton déjà tourné présenté de nouveau révoque toutes les sessions
  (`routes/auth.rs:379`). Le correctif rend le renouvellement fréquent et donc la course réelle, y compris sur le chemin
  401 existant.
- **Retenu** : `navigator.locks.request('kesh:auth:refresh')` autour du renouvellement (à l'entrée **et** sur 401) ; sous le
  verrou, si `localStorage['kesh:auth:renewedAt']` est postérieur ou égal à la décision de l'onglet, `GET /auth/me` au lieu de `POST
  /auth/refresh` ; après succès, écriture de `renewedAt` et message `auth-renewed` sur le canal existant. Sans
  `navigator.locks`, repli sur la seule comparaison de `renewedAt` (course résiduelle déclarée).
- **Écartées** : (a) une période de grâce au serveur pour un jeton tout juste tourné — affaiblit la détection de vol ;
  (b) un onglet « maître » élu — plus complexe, même résultat ; (c) rien — déconnexion de tous les onglets à chaque
  renouvellement simultané.
- **Réversible** : oui.

### C-v0172b-5 — #876 : la recette de session est manuelle, non un E2E

- **Contexte** : la suite E2E partage un backend aux durées par défaut (15 minutes) ; un E2E de session demanderait un
  backend dédié à durées courtes et plusieurs minutes d'attente.
- **Retenu** : Vitest à minuteries simulées pour la logique (AC-A3, A4), tests d'intégration pour le serveur (AC-A1, A2), et
  une recette manuelle à 2/2 minutes sur la base jetable `kesh_v0172b`, déroulé au Dev Agent Record (AC-A5).
- **Écartées** : un projet Playwright dédié à durées courtes — coût récurrent pour un comportement couvert en unitaire.
- **Réversible** : oui.

### C-v0172b-6 — #840 : le verrou nommé du compte, sans migration

- **Contexte** : `create` et `create_csv` (`routes/bank_imports.rs:1157-`, `:2060-`) contrôlent puis insèrent sans
  exclusivité ; l'unicité `(company_id, file_hash)` a été retirée (`20260507000001`) parce que le réimport confirmé est
  légitime.
- **Retenu** : le motif de la saisie manuelle (`bank_statements_manual.rs:211-283`) : `with_account_lock` sur une connexion
  dédiée, la section transactionnelle existante déplacée **telle quelle** sous le verrou, verrou rendu après le commit,
  `MANUAL_STATEMENT_LOCK_TIMEOUT_SECS` réutilisée, `409 RECONCILIATION_ACCOUNT_LOCKED` au délai. L'ordre des refus est
  inchangé.
- **Écartées** : (a) une contrainte d'unicité — migration, et elle interdirait le réimport confirmé ; (b) `SELECT … FOR
  UPDATE` sur la ligne du compte — sérialiserait aussi, mais pas avec la saisie manuelle ni le rapprochement, qui tiennent le
  verrou nommé : deux mécanismes pour une même exclusion ; (c) un verrou en mémoire du processus — faux dès deux instances ou
  un redémarrage.
- **Réversible** : oui (aucun schéma).

### C-v0172b-7 — #870 : pagination et filtre de statut dans l'adresse, sans toucher au serveur

- **Contexte** : le serveur pagine et filtre déjà (`routes/supplier_invoices.rs:640-`, `services/factures_fournisseur.rs:28-30,
  :65, :116`) ; la page ne lit pas l'URL et demande 100 lignes.
- **Retenu** : motif de `invoices/+page.svelte` (`initFromUrl`, `syncUrl`, *Précédent*/*Suivant*) ; `?status=` d'**une** valeur,
  `?offset=`, `?limit=` (50 par défaut, borné à [1, 200]) ; avec un statut, bandeau du filtre et case des annulées masquée ;
  lien K8 vers `?status=draft`.
- **Écartées** : (a) un sélecteur de statut complet qui remplacerait la case des annulées — change l'écran et le manuel de
  la 0.17.0 au-delà du besoin ; (b) un statut multiple dans l'URL — aucun lien ne le demande ; (c) relever la limite à 200 —
  repousse le défaut.
- **Réversible** : oui.

### C-v0172b-8 — #851 : la cause est le débordement de la barre latérale ; reproduire avant de corriger

- **Contexte** : l'issue suppose un rôle momentanément nul ; le code le réfute (remplacement atomique de `_currentUser`,
  garde de `/admin/mcp` qui redirigerait, disparition qui serait totale). La `<nav>` a une hauteur fixe sans `overflow-y`
  (`+layout.svelte:374-376`), pour 33 entrées de 44 px en mode guidé (11 + 4 + 18 ; « 34 » à la rédaction, recompté à la
  validation P1).
- **Retenu** : E2E de reproduction (chaque entrée d'Administration atteignable, 1280 × 720) rouge d'abord, puis
  `overflow-y-auto min-h-0` sur la `<nav>`. Si la reproduction ne rougit pas : aucune correction, constat au rapport.
- **Écartées** : corriger à l'aveugle ; réduire la hauteur des liens (change le mode guidé).
- **Réversible** : oui.

### C-v0172b-9 — #882 : le taux désactivé reste affiché et le refus dit quoi faire ; le serveur ne change pas

- **Contexte** : options du `<select>` limitées aux taux actifs (`InvoiceForm.svelte:54-74`) ; le serveur refuse aussi tout
  taux inactif (`routes/vat.rs`, `verify_vat_rates_against_db`, `active = TRUE`).
- **Retenu** : option `7.70% (désactivé)` pour le taux porté par la ligne, ensemble dérivé des lignes ; message de refus
  explicite — « choisissez un taux actif », **sans** parler de réactivation, qui n'existe pas (`settings/vat-rates/+page.svelte:231-252`) ;
  le repli `8.10` limité aux lignes neuves (C-v0172b-27) ; symétrique de `extraRates` du formulaire fournisseur.
- **Écartées** : accepter un taux inactif au serveur — règle TVA, jalon « Vague 2 » reporté, conséquence sur le décompte non
  établie ; remplacer d'office le taux par le premier actif — changement muet d'un montant de TVA.
- **Réversible** : oui.

### C-v0172b-10 — #880 : `formatSwissDate` tolère l'horodatage ; un helper voisin `formatSwissDateTime`

- **Contexte** : `formatSwissDate` (`reports.api.ts`) rend un résultat faux sur `2026-11-03T17:40:45` (#883.6) ; plusieurs
  sites de #880 affichent un horodatage (`importedAt`, `lastUsedAt`).
- **Retenu** : `formatSwissDate` reconnaît `AAAA-MM-JJ` suivi de `T`, d'une espace ou de rien ; `formatSwissDateTime` rend
  `JJ.MM.AAAA HH:mm` ; aucun fuseau appliqué (horodatages naïfs, affichés tels quels aujourd'hui) ; `api-keys` délègue.
- **Écartées** : `Intl.DateTimeFormat` — forme dépendante de la langue, alors que les écrans de la 0.17.1 ont fixé
  `JJ.MM.AAAA` ; un `.slice(0, 10)` à chaque site — duplication, et l'heure perdue là où elle est affichée aujourd'hui.
- **Réversible** : oui.

### C-v0172b-11 — #880 : l'`aria-label` des propositions de rapprochement est inclus

- **Contexte** : `ReconciliationProposals.svelte:461, :471` passent `bookingDate` brute au message lu par un lecteur d'écran.
- **Retenu** : même forme suisse que la cellule voisine (`:480`) — ce qui est lu doit dire ce qui est affiché.
- **Écartées** : n'y toucher que pour l'affichage visible.
- **Réversible** : oui.

### C-v0172b-12 — #880 : `UnrejectTransactionAction` reste en `Intl`

- **Contexte** : `UnrejectTransactionAction.svelte:39` formate par `Intl.DateTimeFormat(i18nLocale(), { dateStyle: 'medium' })`.
- **Retenu** : conforme — ce n'est pas l'ISO que vise #880 ; l'uniformiser est un choix de présentation hors de l'issue.
- **Écartées** : le passer à `formatSwissDate` — change un texte localisé voulu (C-28-5-15).
- **Réversible** : oui.

### C-v0172b-13 — #883.2 : sans objet

- **Contexte** : « des lignes Svelte dépassent probablement la largeur Prettier ». Le dépôt n'a ni Prettier ni
  configuration de largeur (`frontend/package.json`, `git ls-files`).
- **Retenu** : rien ne change ; l'orchestrateur le dit à l'issue.
- **Écartées** : refluer à la main sans règle — bruit de diff sans critère ; introduire Prettier — hors périmètre d'une
  corrective (reformaterait tout le frontend).
- **Réversible** : oui.

### C-v0172b-14 — #883.4 : un payload précédé d'un BOM reste refusé, et le test le fige

- **Contexte** : `est_payload_spc` (`qr_decode.rs:39-41`) teste `starts_with` ; un BOM en tête le fait échouer.
- **Retenu** : figer le comportement actuel (refus) : la norme SIX n'admet pas de BOM, et aucune pièce connue n'en porte.
- **Écartées** : retirer un BOM avant le test — élargit sans cas réel.
- **Réversible** : oui.

### C-v0172b-15 — #885 : la date « à vérifier » ne bloque pas l'enregistrement

- **Contexte** : #853 venait d'une date du jour restée **sans le dire** ; #885 la veut proposée et visible.
- **Retenu** : avis visible jusqu'à modification ou confirmation (*C'est la bonne date*), sans blocage ; date du jour
  **locale** (`todayLocal()`), non UTC.
- **Écartées** : bloquer tant que non confirmée — un clic obligatoire de plus, que l'issue n'a pas demandé ; garder le champ
  vide — refusé par le Project Lead.
- **Réversible** : oui (une condition à ajouter dans `onSubmit`).

### C-v0172b-16 — #885 : l'échéance suit la date tant qu'elle n'a pas été saisie, à la création seulement

- **Contexte** : le formulaire fournisseur n'emploie pas les conditions de paiement (prémisse de l'issue fausse) ;
  `addDaysIso` et `defaultPaymentTermsDays` existent (#245, facture cliente).
- **Retenu** : à la création (avec ou sans document), échéance = date + délai du fournisseur choisi, ou la date ; recalculée
  à chaque changement de date ou de fournisseur tant que l'utilisateur ne l'a pas saisie ; à l'édition d'un brouillon,
  jamais recalculée.
- **Écartées** : calculer au serveur — l'utilisateur ne la verrait qu'après enregistrement ; recalculer à l'édition —
  écraserait une échéance enregistrée.
- **Réversible** : oui.

### C-v0172b-17 — CHANGELOG commun à la 0.17.2

- **Contexte** : la story `v0172-annulees` (#868) écrit aussi la section `## [0.17.2] — Non publié`.
- **Retenu** : puces de cette story sous *Corrigé* / *Ajouté* ; fusion par union à l'intégration ; l'intro est vérifiée contre
  l'autre story (une migration de sa part changerait « retour sans restauration »).
- **Écartées** : une section par story.
- **Réversible** : oui.

### C-v0172b-18 — #876 : `/me` lit la ligne du jeton de renouvellement, sans écriture

- **Contexte** : après un rechargement, seul `/me` peut dire l'échéance du jeton de renouvellement.
- **Retenu** : `find_active_by_token` sur le cookie `kesh_refresh_token` (chemin `/api/v1/auth`, envoyé à `/me`) ; `null`
  sans cookie, inconnu, révoqué ou expiré ; aucune mise à jour de la ligne.
- **Écartées** : prolonger la ligne à chaque `/me` — écriture par lecture, et `/me` est appelé au démarrage de chaque onglet.
- **Réversible** : oui.

### C-v0172b-19 — Issue demandée : la sélection d'un lot de paiement est limitée à 200 factures validées

- **Contexte** : `routes/(app)/payment-batches/+page.svelte:54` (`listSupplierInvoices({ limit: 200, status: ['open'] })`),
  même famille que #870, non mécanique (sélecteur paginé ou recherche).
- **Retenu** : hors périmètre ; l'orchestrateur ouvre l'issue (« on trace toujours »).
- **Réversible** : —

### C-v0172b-20 — Issue demandée : la « date du jour » calculée en UTC

- **Contexte** : `….toISOString().slice(0, 10)` rend la veille entre minuit et 1 h (2 h en été) en Suisse. Le motif large
  `git grep -nE '\.toISOString\(\)\.slice\(0, ?10\)' -- frontend/src ':!*.test.ts'` rend **16 lignes** (relevé à la validation
  P1 ; la première rédaction n'en comptait que dix, par un motif qui manquait les formes en deux temps) : `InvoiceForm.svelte:110`,
  `SettleInvoiceDialog.svelte:77`, `WriteOffDialog.svelte:64`, `invoice-helpers.ts:124`, `journal-entries/form-helpers.ts:63`,
  `ManualReminderDialog.svelte:46`, `reminders/reminder-sent-at.ts:24`, `reports.api.ts:328`, `SupplierInvoiceForm.svelte:153`
  (corrigé par #885), `invoices/[id]/+page.svelte:106`, `invoices/due-dates/+page.svelte:297`, `payment-batches/+page.svelte:37`,
  `payment-batches/[id]/+page.svelte:26`, `settings/+page.svelte:219`, `settings/api-keys/+page.svelte:51`,
  `supplier-invoices/[id]/+page.svelte:90`. Certaines sont du calcul de dates (à trier), non une « date du jour ».
- **Retenu** : seul le site de #885 passe à `todayLocal()` ; l'issue demandée à l'orchestrateur porte les 15 autres lignes, à trier.
- **Écartées** : les corriger tous ici — des écrans sans lien avec les issues de la story.
- **Réversible** : —

### C-v0172b-21 — Aucune migration ; retour à la 0.17.1 sans restauration

- **Contexte** : contrainte de production (`v017-donnees-a-garder.md`) ; consigne de préférer une solution sans migration.
- **Retenu** : verrou nommé (#840), champ JSON additif (#876), tout le reste au frontend ; 88 migrations avant et après (`ls crates/kesh-db/migrations/*.sql | wc -l` ; « 86 » à la
  rédaction, faux, corrigé à la validation P1).
- **Écartées** : contrainte d'unicité pour #840 (C-v0172b-6).
- **Réversible** : —

### C-v0172b-22 — Les textes neufs passent par l'i18n, quatre locales

- **Contexte** : `InvoiceForm.validate` écrit ses messages en français en dur (`:480-497`) ; les écrans fournisseur et la
  liste emploient `i18nMsg`.
- **Retenu** : chaque texte neuf (bandeau du filtre, pagination fournisseur, libellé « (désactivé) » et son refus, avis « à
  vérifier » et son bouton, lien K8) a sa clé dans les quatre `messages.ftl` ; les messages voisins en dur de `validate`
  restent tels quels.
- **Écartées** : passer tout `validate` à l'i18n — hors périmètre.
- **Réversible** : oui.

### C-v0172b-23 — #876 : pas de minuterie ; l'entrée renouvelle, au plus une fois par pas `T`

- **Contexte** : validation P1, findings P1 et P2 — une minuterie qui renouvelle « s'il y a eu activité depuis le dernier
  renouvellement » laisse la session vivre jusqu'à `2N − 1` après la dernière entrée, et ne sonne pas pendant une veille ni à
  l'heure dans un onglet masqué.
- **Retenu** : à chaque entrée (clavier, pointeur, molette, toucher, retour sur l'onglet), l'horloge murale est relue ; si la
  session vit et que le dernier renouvellement (de cet onglet ou d'un autre) date d'au moins `T = min(60 s, N/4, jwt/4)`, on
  renouvelle. Borne d'inactivité : entre `max(N, jwt) − T` et `max(N, jwt)` après la dernière entrée (14 à 15 minutes par
  défaut). Coût : au plus un renouvellement par minute d'activité et par utilisateur.
- **Écartées** : (a) la minuterie de la première rédaction ; (b) une minuterie de secours en plus de l'entrée — inutile : sans
  entrée, il n'y a rien à prolonger ; (c) un pas fixe de 60 s — trop long pour une inactivité réglée à une minute.
- **Réversible** : oui.

### C-v0172b-24 — #876 : un échec transitoire du renouvellement anticipé ne déconnecte pas

- **Contexte** : `doRefresh` efface la session sur toute erreur (`api-client.ts:96-99, :174-178`), défendable sur 401 (session
  morte), faux quand la session vit encore (finding P3).
- **Retenu** : sur le chemin anticipé, seul `401 INVALID_REFRESH_TOKEN` efface la session ; erreur réseau, 429, 5xx : la
  session reste, l'entrée suivante retente au plus tôt 10 s plus tard tant que la session vit. Le chemin 401 garde son
  comportement, plus la garde « 401 sans entrée depuis le dernier renouvellement : ni renouvellement ni effacement » (sans
  elle, une sonde de fond prolongerait la session dès que l'inactivité réglée dépasse la durée du jeton d'accès).
- **Écartées** : réessayer en boucle sans entrée — prolongerait une session inactive ; effacer sur toute erreur — la perte de
  saisie de #876 par un autre chemin.
- **Réversible** : oui.

### C-v0172b-25 — Limites déclarées, et issue demandée : le jeton de renouvellement stocké en clair

- **Contexte** : (a) l'import tient deux connexions d'un pool de 5 (`main.rs:83`) ; (b) au-delà de 5 s sous le verrou, le
  second envoi rend le `409` du rapprochement ; (c) en refermant l'inventaire du manuel d'administration, `admin-manual.tex:2996`
  affirme un hash SHA-256 du jeton de renouvellement, alors qu'il est stocké tel quel (`repositories/refresh_tokens.rs:30-34`,
  `.bind(&new.token)`).
- **Retenu** : (a) et (b) déclarés en Dev Notes, non corrigés (même risque que la saisie manuelle, déjà accepté) ; (c) le manuel
  est corrigé (il dit ce que fait le code) ; le stockage en clair, défaut de sécurité sans lien avec #876, fait l'objet d'une
  **issue demandée** (« on trace toujours »).
- **Écartées** : hacher le jeton dans cette story — touche le dépôt et les sessions ouvertes, hors périmètre d'une corrective.
- **Réversible** : —

### C-v0172b-26 — #870 : l'URL est la source de vérité, lue de façon réactive ; le filtre ne porte que sur le statut

- **Contexte** : validation P1, finding P6 — le motif `initFromUrl` à `onMount` ne relit pas l'URL quand seule la requête
  change ; un lien vers `/supplier-invoices` depuis `?status=draft` laisserait la liste filtrée. Finding P20 : K8 compte les
  brouillons de l'exercice, le lien ouvre ceux de tous les exercices.
- **Retenu** : `status`, `offset`, `limit` dérivés de `page.url` ; les gestes naviguent ; une page au-delà du total ramène à la
  dernière page ; le bandeau, le manuel et le CHANGELOG disent « de tous les exercices ».
- **Écartées** : filtrer aussi par dates depuis K8 — l'issue ne le demande pas, et un sur-ensemble suffit à atteindre chaque
  brouillon ; faire du lien du bandeau un bouton qui remet l'état — le lien du menu resterait sans effet.
- **Réversible** : oui.

### C-v0172b-27 — #882 : le repli `8.10` ne touche plus que les lignes neuves

- **Contexte** : `InvoiceForm.svelte:83-92` réécrit en silence toute ligne à `8.10` quand ce taux n'est plus actif, lignes
  relues comprises (finding P9) — exactement ce que C-v0172b-9 écarte.
- **Retenu** : une marque d'interface `_vatPlaceholder` sur les seules lignes créées avec la valeur provisoire avant le
  chargement des taux (retirée par `stripUiKey`) ; le repli ne touche qu'elles ; une ligne relue ou venue d'un article garde
  son taux, affiché « (désactivé) » s'il est inactif.
- **Écartées** : retirer le repli — une ligne neuve créée avant le chargement garderait `8.10` même absent de la société ;
  le garder tel quel — changement muet de TVA.
- **Réversible** : oui.
