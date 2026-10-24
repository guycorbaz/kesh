# Epic 32 — registre des choix autonomes de la Story 32-1 (défauts courts, clés 32-1a, 32-1b, 32-1c)

Registre propre à la story, à verser **par union** dans `epic-32-choix-autonomes.md` au merge
(`docs/agents-orchestration.md` § 1). Fiche : `32-1-defauts-courts.md`. Clé courte : `321`.
Rédigé le 2026-10-24 (Opus 5.5), worktree `kesh-32-1`, branche `story/32-1` sur `chore/kickoff-epic-32` (`6f0cee00`).

---

### C-32-1-1 — Une fiche, trois clés, un registre

- **Contexte** : C-32-0-3 fixe une fiche pour trois clés (32-1a, 32-1b, 32-1c), comme la 31-1.
- **Retenu** : `32-1-defauts-courts.md`, trois parties A, B, C, numérotation des AC propre à chaque partie (`AC-a1`,
  `AC-b1`, `AC-c1`…) pour qu'une revue d'une clé ne lise que sa partie ; un seul registre (`-321`) ; un seul prompt de
  validation P1, qui couvre les trois parties.
- **Écartées** : trois fiches (trois fois l'en-tête, l'origine, les décomptes) ; une numérotation continue (une revue de
  la 32-1b citerait des AC de la 32-1a).
- **Réversible** : oui.

### C-32-1-2 — La « M4 de la 31-5 » reste équivalente ; c'est la mutation du `COALESCE` qui doit être tuée

- **Contexte** : le fichier d'epic écrit pour la 32-1a « les mutations M4 et M9 de la 31-5 sont **tuées** ». Or la M4 de
  la 31-5 (`31-5-defauts-courts.md`, tableau des mutations) est `if invoice_settings_created` → `if true` dans
  `final_body` (`crates/kesh-seed/src/lib.rs`), déclarée **équivalente** avec sa raison : à l'étape 2 la ligne de
  réglages est toujours neuve, et `COALESCE` n'écraserait rien. Le constat E1 de #716 dit autre chose : la mutation qui
  remplace `COALESCE(col, ?)` par `?` dans `designate_vat_accounts_in_tx`
  (`crates/kesh-db/src/repositories/company_invoice_settings.rs:1321-1323`) **survivrait** — « (M4 équivalente) »
  désigne le motif, non la même mutation. Un test de dépôt qui pose une valeur avant l'appel tue la mutation du
  `COALESCE` ; il ne tue pas la M4 de la 31-5, qui ne passe pas par une ligne préexistante.
- **Retenu** : la 32-1a tue **M-a1** (le `COALESCE` d'une colonne remplacé par `?`) et **M-a2** (`latestLaterClosed`
  sans son filtre `status === 'Closed'`, la M9 de la revue de la 31-5) ; la M4 de la 31-5 reste **équivalente**, et la
  fiche le dit. Prémisse du fichier d'epic corrigée (à signaler à l'orchestrateur).
- **Écartées** : chercher un test qui « tue » la M4 de la 31-5 (il faudrait une ligne de réglages préexistante dans
  `seed_demo`, ce que la démonstration ne produit pas : un tel test prouverait un cas qui n'existe pas).
- **Réversible** : oui.

### C-32-1-3 — B3 : un test d'existence, pas de repli Rust, pour `invoice-qr-message-prefix`

- **Contexte** : `i18n.format(&locale, "invoice-qr-message-prefix", None)` (`invoice_pdf_service.rs:240`) rend la clé
  brute si elle manque dans les quatre catalogues (`crates/kesh-i18n/src/loader.rs:109-125`), et cette clé brute entrerait
  dans la communication de la QR.
- **Retenu** : un test unitaire dans `invoice_pdf_service.rs` (`mod tests`, à côté de
  `reminder_fees_note_fits_its_width_in_all_four_locales`, qui charge déjà les catalogues réels) : pour chacune des
  quatre locales, la valeur n'est ni vide ni la clé ; pour `de-CH`, `it-CH`, `en-CH`, elle diffère de `fr-CH`.
  **Aucun repli Rust** : un repli rendrait silencieux le retrait de la clé, exactement ce qu'on veut voir rougir.
  `kesh-i18n` n'est donc pas touché.
- **Écartées** : un repli `"Facture"` (masque le défaut au lieu de le signaler, et en français dans une QR allemande) ;
  l'ajout de la clé à G9 (G9 compare des replis existants, il n'en crée pas).
- **Réversible** : oui.

### C-32-1-4 — E-9 (1) : une sonde dans `TestEnv` ; la distinction création / lecture du dossier est équivalente

- **Contexte** : `dossier_auto_inutilisable` (`admin_auto_backup_e2e.rs:684`) n'assert que `failed`. Établi au code
  (`auto_backup.rs:589-600`, dans `ecrire`, l. 577) : deux gardes successives (création `DirBuilder`, puis `read_dir`) rendent
  `failed` ; tout chemin où la création échoue (fichier à la place du dossier, parent non inscriptible) fait aussi
  échouer `read_dir`. Ignorer l'erreur de création est donc une mutation **équivalente** quant au statut.
- **Retenu** : `TestEnv` compte ses appels à `free_bytes` et `wrap_writer` ; le test assert **zéro** appel à chacun —
  le tour s'est arrêté à une garde du dossier, avant la mesure d'espace et avant toute écriture. Mutation **M-a3** :
  neutraliser les **deux** `return` des gardes du dossier (le tour continue jusqu'à `write_backup_file`, qui échoue
  aussi en `failed`) → la sonde rougit. Le doc-comment du test dit l'équivalence de l'ordre création / lecture.
- **Écartées** : capturer le journal (`tracing`) pour lire le message de la garde (aucune capture n'existe dans les tests
  de `kesh-api`, et ce serait la seule) ; réécrire `ecrire` pour rendre l'étape qui échoue (une modification de
  production pour un test).
- **Réversible** : oui.

### C-32-1-5 — E-9 (2) : la garde lexicale de l'export reste la seule, et le dit

- **Contexte** : le garde de `full_export` (`routes/admin.rs:77-80`) est déplacé dans la fermeture `remplir` pour vivre
  dans le `spawn_blocking` « quel que soit le sort du client ». Sans ce déplacement (M-c24 de la 31-1c), le garde vit
  jusqu'à la sortie du handler : la différence ne s'observe que si le **client part pendant l'assemblage** (le handler
  est abandonné, le verrou rendu, le `spawn_blocking` écrit encore). Un test comportemental exige un crochet de
  synchronisation dans l'assemblage — conception, non mécanique.
- **Retenu** : le doc-comment de `gardes_possedes` (`auto_backup.rs:1190-1201`) dit pourquoi la garde lexicale est la
  seule (propriété observable au seul départ du client) et que c'est un angle mort assumé. Aucun test neuf.
- **Écartées** : un test qui abandonne une requête d'export en vol (dépend du temps d'assemblage, donc flaky) ; un
  crochet de test dans `build_keshbackup` (production modifiée pour un test, hors du périmètre court).
- **Réversible** : oui (le crochet peut venir avec E-1, qui touche le même verrou).

### C-32-1-6 — E-9 (3) : `code()` lit les commentaires de bloc, `argument_de` les littéraux ; `corps()` documenté

- **Contexte** : `code()` (`auto_backup.rs:1121`) n'écarte que les lignes `//` ; `argument_de` (`:1159`) apparie les
  parenthèses sans voir les chaînes ; `corps()` (`:1182`) coupe à la première ligne `}` en colonne 0.
- **Retenu** : `code()` retire aussi les commentaires `/* … */` (imbriqués, comme en Rust) ; `argument_de` ignore les
  parenthèses dans `"…"` (échappements compris) et dans les littéraux `'('` et `')'` ; `corps()` garde sa règle, qui
  repose sur rustfmt (un item de tête se ferme par `}` en colonne 0), et la dit, avec une assertion de montage. Chaque
  helper reçoit son assertion de montage (cas qui rougit avant le correctif).
- **Écartées** : un analyseur Rust (`syn`) pour trois gardes lexicales (une dépendance pour des tests) ; laisser les
  helpers tels quels (le constat est réel : un `/* auto_backup::spawn(x) */` dans `main.rs` compterait).
- **Réversible** : oui.

### C-32-1-7 — E7 : le doc-comment de `kesh-db` seul ; `kesh-seed` n'est pas touché

- **Contexte** : E7 vise le doc-comment de `lock_vat_role_accounts_in_tx`, qui décrit l'ordre des trois verrous de TVA
  sans dire que la transaction complète de `final_body` (1171, 2200, 2206, puis les rôles de
  `insert_with_defaults_in_tx` : `AccountRole::Receivable`, `DefaultRevenue`, `Payable`, trois lectures `FOR UPDATE`,
  `company_invoice_settings.rs:1074-1105`) n'est pas croissante au plan standard : le compte
  `Receivable` (1100) et le compte `Payable` (2000) y viennent **après** 2206 — à relire au plan livré par le
  développement, qui cite les identifiants constatés. Le doc-comment de `final_body`
  (`crates/kesh-seed/src/lib.rs:364-371`) écrit déjà la séquence `accounts (TVA, puis rôles de
  insert_with_defaults_in_tx)` ; il est juste.
- **Retenu** : une phrase de plus au seul doc-comment de `lock_vat_role_accounts_in_tx` : la séquence complète de la
  transaction n'est pas croissante (les rôles d'`insert_with_defaults_in_tx` viennent après, dont deux de numéros plus
  bas au plan standard), et le rejeu de `seed_demo` sur interblocage (`is_seed_retryable`) en est le filet. La 32-1a reste
  à trois modules.
- **Écartée** : retoucher aussi `final_body` (un cinquième site pour une phrase déjà juste).
- **Réversible** : oui.

### C-32-1-8 — E-6 : une saisie non enregistrée survit à « Sauvegarder maintenant » ; « Recharger » la remplace

- **Contexte** : `runNow` (`AutoBackupSection.svelte:139-153`) passe la réponse à `apply` (`:63-69`), qui réécrit
  `enabled`, `intervalHours`, `retentionCount`. Les écritures de la tâche (`record_*`) ne touchent jamais `version`
  (`crates/kesh-db/src/repositories/backup_schedule.rs:9-15`) : la version lue reste valable après un run.
- **Retenu** : une fonction `applyState(value)` — l'état (`schedule`, état partagé) toujours, les trois champs du
  formulaire **seulement s'ils égalent encore le réglage lu** (formulaire non modifié). `runNow` et la relecture de
  l'AC-b2 l'emploient ; `loadSchedule` (montage, « Recharger » après un 409) et `save` gardent `apply` (remplacement
  complet, voulu).
- **Écartées** : un avertissement avant « Sauvegarder maintenant » (un dialogue de plus pour un geste fréquent) ;
  enregistrer d'office la saisie avant le run (une écriture que l'utilisateur n'a pas demandée).
- **Réversible** : oui.

### C-32-1-9 — E-7 : tout échec de « Sauvegarder maintenant » autre qu'un 409 `BACKUP_IN_PROGRESS` relit l'état

- **Contexte** : le délai client de `runBackupNow` est `LONG_OPERATION_TIMEOUT_MS` (60 min,
  `frontend/src/lib/shared/utils/api-client.ts:53`) ; mais un proxy coupe bien avant (Nginx `proxy_read_timeout 300s`,
  manuel § proxy), et rend un 502/504 ; un `record_success` en erreur rend 500 alors que l'archive est écrite (E-2).
  Dans tous ces cas, l'écran dit « n'a pas pu être lancée » sans relire, alors que la sauvegarde a pu être écrite.
- **Retenu** : sur tout échec **autre que** 409 `BACKUP_IN_PROGRESS` (dont le message serveur reste affiché tel quel),
  l'écran affiche un message neuf — la réponse n'est pas parvenue, la sauvegarde a pu être écrite, l'état et la liste
  ont été relus — et relit l'état (`applyState`, C-32-1-8) et la liste. Clé neuve `admin-backup-auto-run-unknown`.
- **Écartées** : distinguer `TIMEOUT` / `NETWORK_ERROR` des 5xx (le résultat est inconnu dans les deux cas) ; sonder
  l'état jusqu'à voir changer `lastAttemptAt` (une boucle pour un cas rare).
- **Réversible** : oui.

### C-32-1-10 — E-8 : un échec de sondage garde l'état connu ; une réponse tardive est ignorée

- **Contexte** : `refreshBackupScheduleState` (`backup-schedule.svelte.ts:30-36`) pose `null` sur tout échec, et une
  réponse lancée avant `setBackupScheduleState` peut l'écraser. La fiche de la 31-1c (AC31) dit « un échec de cette
  lecture n'affiche aucun bandeau » : vrai d'une **première** lecture (l'état est `null`) ; l'éteindre sur un échec
  passager n'en découle pas.
- **Retenu** : un échec de lecture **laisse** l'état tel qu'il est (s'il est `null`, aucun bandeau : le test « lecture en
  échec : absent » reste vrai) ; un compteur de séquence, incrémenté par `setBackupScheduleState` et par chaque
  lecture, fait ignorer toute réponse dont la séquence n'est plus la dernière (motif du jeton de requête de C-29-1-37,
  `fiscal-years`). Le doc-comment du composant et le manuel (§ *Le bandeau de l'application*) le disent.
- **Écartées** : garder l'état connu avec une date d'expiration (une horloge du navigateur, que le module s'interdit) ;
  un bandeau « état inconnu » (bruit sur chaque coupure réseau).
- **Réversible** : oui.

### C-32-1-11 — L-B5 : le serveur dit qu'une réussite est datée du futur, le bandeau le dit en clair

- **Contexte** : `is_overdue` est vrai pour une réussite postérieure à `now` (`auto_backup.rs:221-226`, voulu : horloge
  revenue en arrière) ; le bandeau affiche alors « Aucune sauvegarde automatique réussie depuis le <date future> »
  (`BackupScheduleBanner.svelte:33`, clé `admin-backup-banner-overdue`). Le navigateur ne compare aucune date à sa propre
  horloge (doctrine du module, en-tête de `backup-schedule.api.ts`) : la distinction doit venir du serveur.
- **Retenu** : une fonction pure `is_success_in_future(schedule, now)` dans `auto_backup.rs` (vrai si
  `last_success_at` existe et que `passe()` l'écarte), un champ `lastSuccessInFuture: bool` dans
  `BackupScheduleResponse` (`routes/backup_schedule.rs`) ; le bandeau, dans ce cas, affiche la clé neuve
  `admin-backup-banner-future` (la dernière réussite est datée du { $date }, après l'heure du serveur : son horloge a
  reculé). `bannerKind` ne change pas (le bandeau reste `overdue`, `data-kind="overdue"`).
- **Écartées** : masquer la date (le libellé « depuis le » sans date est absurde aussi) ; ne pas allumer le bandeau (le
  manuel, § *Horloge du serveur déréglée*, dit qu'il s'allume, et c'est utile : l'horloge est fausse) ; comparer dans le
  navigateur (contraire à la doctrine du module).
- **Réversible** : oui (un champ additionnel de réponse).

### C-32-1-12 — L-B6 : la mention garde les Mio de la variable et ajoute l'équivalent décimal

- **Contexte** : la ligne affiche sa taille en unités décimales (`formatBackupSize`, `backup-format.ts:14-28`) et la
  mention « limite N Mio » compare à `importMaxMib × 1 048 576` (`backup-schedule.api.ts:130`). Les Mio sont l'unité de
  `KESH_ADMIN_IMPORT_MAX_MB`, celle que l'administrateur relève.
- **Retenu** : `admin-backup-server-too-large` prend une variable de plus, `{ $size }` = la limite formatée par
  `formatBackupSize` : « trop grande pour être restaurée par l'écran (limite { $mib } Mio, soit { $size }) » ; quatre
  locales ; le manuel (§ *Les sauvegardes du serveur*) cite la nouvelle forme.
- **Écartées** : afficher les tailles en unités binaires (`Intl.NumberFormat` n'a pas d'unité `mebibyte`, et toutes les
  autres tailles de l'écran changeraient) ; n'afficher que l'équivalent décimal (l'administrateur ne saurait plus quelle
  valeur de la variable relever).
- **Réversible** : oui.

### C-32-1-13 — L-B7 : le refus traduit, prouvé dans `i18n_e2e.rs`

- **Contexte** : `update_backup_schedule` rend `AppError::Validation("Réglage de sauvegarde hors domaine : …".into())`
  (`routes/backup_schedule.rs:133-137`), en français dans les quatre langues. Le refus du `PUT` est servi par un bloc
  `admin_routes` ; `i18n_e2e.rs` monte déjà l'application dans une locale donnée (`spawn_app_with_locale`, l. 65) avec un
  administrateur (`ensure_admin_user`) et son propre verrou de l'état i18n global (l. 25-32).
- **Retenu** : `crate::errors::t("admin-backup-schedule-error-out-of-domain", "<valeur fr-CH>")`, la clé dans les quatre
  catalogues, une entrée de plus à la table de G9 (`textes_coherents.rs`, `les_replis_rust_suivent_le_catalogue`), et un
  test neuf dans `i18n_e2e.rs` : en `de-CH`, `PUT /api/v1/admin/backup-schedule` avec `intervalHours: 7` rend 400, un
  message égal au rendu `de-CH` de la clé et différent du rendu `fr-CH`.
- **Écartées** : le test dans `admin_auto_backup_e2e.rs` (ses autres tests supposent le français ; un `init_error_i18n`
  en `de-CH` y ferait dériver leurs messages sous `cargo test`, processus partagé) ; dupliquer `spawn_app_de` de
  `audit_log_e2e.rs` (règle DRY, et c'est le constat B2).
- **Réversible** : oui.

### C-32-1-14 — E-1 et E-2 : une seule issue, deux parties

- **Contexte** : E-1 (verrou tenu sans limite par une écriture figée ; aucun bandeau sans réussite antérieure) et E-2
  (`record_success` en erreur après l'écriture) demandent chacun une conception — un délai et ce qu'il fait d'une
  écriture en cours ; le chemin d'erreur d'une archive écrite mais non enregistrée (rétention, réponse, reprise). Établi
  au code : `run_once` prend le verrou en `try_lock_owned` (`auto_backup.rs:669`) et le garde jusqu'à la fin ; export et
  import l'attendent en `lock_owned().await` (`routes/admin.rs:77`, `:213`) ; `is_overdue` est faux sans
  `last_success_at` (`auto_backup.rs:221-226`) ; `record_success` (`:713`) en `Err` remonte par `?` (aucune rétention,
  500 au `POST …/run`, `last_attempt_at` déjà posé : reprise dans une heure, seconde archive).
- **Retenu** : une issue « robustesse du tour de sauvegarde automatique », deux parties, texte prêt dans la fiche
  (*Issue à ouvrir*) ; l'orchestrateur peut la scinder.
- **Écartées** : les intégrer à la 32-1b (conception d'un délai et d'un chemin d'erreur, non mécanique — C-32-0-3) ; deux
  issues (même module, même verrou, même revue probable).
- **Réversible** : oui.

### C-32-1-15 — 32-1c : une phrase se corrige ici ; un geste se renvoie à Guy

- **Contexte** : le fichier d'epic : « la fiche ne rejoue pas la procédure, elle corrige des phrases ; toute correction
  qui changerait un geste se renvoie à Guy ». Les deux choix de procédure de #714 sont tranchés (commentaire de Guy du
  2026-10-24, versés en v0.15.0 : vérifié au manuel, `docs/manual/fr/admin-manual.tex:1791` — l'écart est un arrêt — et
  `:1906` — le `pull` en échec, le troisième listing).
- **Retenu** : la règle de partage, décidable : **la 32-1c** corrige ce qui est faux ou incomplet dans ce que la
  procédure **dit** — une cause nommée, une sortie attendue, une source d'écriture à éviter, une limite d'un contrôle —
  sans ajouter, retirer ni changer **aucune commande** ni **aucune consigne d'agir** ; **tout le reste** (une commande
  changée, une consigne neuve : refaire, refuser, effacer, remplacer) va dans une issue plus étroite, pour Guy (texte
  prêt dans la fiche). Une fois la 32-1c fusionnée et l'issue ouverte, #714 n'a plus de constat propre.
- **Écartées** : tout corriger dans la 32-1c (la procédure n'a été jouée qu'une fois, dans une rédaction antérieure :
  une commande changée sans rejeu est une procédure non éprouvée de plus) ; tout renvoyer à Guy (des phrases fausses
  resteraient au manuel tant qu'il n'a pas tranché).
- **Réversible** : oui.

### C-32-1-16 — Le glyphe ⚠️ remplacé par une macro, dans les deux manuels FR (défaut voisin mécanique)

- **Contexte** : #714 relève que ⚠️ ne s'imprime pas dans le PDF du manuel d'administration (16 lignes du `.tex`).
  Vérifié : `pdftotext docs/manual/fr/admin-manual.pdf - | grep -c "⚠"` rend 0 ; la police de corps (TeX Gyre Heros,
  `docs/manual/shared/kesh-style.sty:95`) n'a pas ce glyphe. Le manuel utilisateur porte le même défaut :
  `grep -c "⚠" docs/manual/fr/user-manual.tex` rend **9**, son PDF 0. La brochure : 0. `kesh-style.sty:37` fournit déjà
  `\faExclamationTriangle` (repli `\textbf{!}`), employé par l'encadré `keshwarning`.
- **Retenu** : une macro `\keshAttention` dans `kesh-style.sty`, définie sur `\faExclamationTriangle`, et les 16 + 9
  occurrences remplacées (correction mécanique, même cause, intégrée comme défaut voisin) ; contrôle : `grep -c "⚠"` sur
  les deux `.tex` rend 0.
- **Écartées** : une police de repli à emoji (`fontspec` + police couleur : dépendance de la chaîne TeX du poste et de la
  CI) ; le seul manuel d'administration (le même défaut resterait, mécanique, dans l'autre).
- **Réversible** : oui.

### C-32-1-17 — Ordre de fusion : 32-1c, puis 32-1b ; le second régénère le PDF

- **Contexte** : la 32-1b et la 32-1c modifient toutes deux `docs/manual/fr/admin-manual.tex` (sections distinctes,
  fusion textuelle sans conflit) et son PDF (binaire : conflit à toute fusion parallèle).
- **Retenu** : la 32-1c (doc seule, plus courte) se fusionne d'abord ; la 32-1b se met à jour sur `main` avant sa revue
  finale et **régénère** le PDF (`make fr`) sur le `.tex` fusionné. La 32-1a ne touche pas le manuel.
- **Écartée** : une PR unique pour les trois clés (trois revues dans une PR, contraire à C-32-0-3).
- **Réversible** : oui (l'ordre inverse marche, en inversant la régénération).

### C-32-1-18 — Gates : complet pour 32-1a et 32-1b, E2E complet au dernier commit de code de chacune ; 32-1c ciblé

- **Contexte** : la 32-1a touche un test de `kesh-db` (exception `kesh-db` du `CLAUDE.md`) ; la 32-1b change du code
  servi (frontend, route) ; la 32-1c ne touche que le manuel et deux fiches — mais trois gardes de `kesh-api` lisent le
  manuel (`textes_coherents.rs`, `configuration_transmise.rs`).
- **Retenu** : 32-1a et 32-1b : gate complet (`scripts/test-fast.sh` sous verrou, base remise à zéro d'abord), les quatre
  gates frontend, **E2E complet au dernier commit de code** (la 32-1a ne change aucune ligne servie, mais le `CLAUDE.md`
  ne fait pas d'exception pour un commit de tests ; l'E2E est le moins cher des arguments). 32-1c : gate ciblé
  `binary(textes_coherents)` et `binary(configuration_transmise)`, `make fr`, contrôle aplati du PDF ; pas d'E2E (aucun
  code).
- **Écartée** : pas d'E2E pour la 32-1a (défendable, mais une revue le contesterait ; le coût d'une contestation est
  supérieur au coût du run).
- **Réversible** : oui.

### C-32-1-19 — Prémisses des issues corrigées au code

- **Contexte** : chaque constat a été relu au site cité, le 2026-10-24, sur `6f0cee00`.
- **Retenu** : les corrections, écrites dans la fiche (§ *Constats établis au code*) et à signaler sur les issues par
  l'orchestrateur : #711 E-7 cite `backup-schedule.api.ts:111` — c'est `downloadServerBackup` ; `runBackupNow` est aux
  l. 95-100 ; #711 L-B3 cite `auto_backup.rs:490-491` — la fonction du seuil est `seuil_d_espace`, l. 496-500 ; #711
  E-9 (1) — « `failed` est aussi le statut de repli de `build_keshbackup` en `Err` » est juste, mais la seule
  distinction **observable** est l'absence de mesure d'espace et d'écriture (C-32-1-4) ; #714 « laissez le dossier vide
  laisse un reste qui bloque une reprise le même jour » — faux à la lettre : la reprise se fait dans un **nouveau**
  dossier (suffixe), que le `mkdir` du contrôle 3 crée sans conflit ; le reste est un dossier vide, inoffensif, que
  `/var/tmp` garde trente jours ; #716 E6 cite `textes_coherents.rs:276,295` — la regex est l. 277-278, son emploi
  l. 296 ; le fichier d'epic, pour la 32-1a, « M4 de la 31-5 tuée » (C-32-1-2).
- **Réversible** : sans objet (relevé).
