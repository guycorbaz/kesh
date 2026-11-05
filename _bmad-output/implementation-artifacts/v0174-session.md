# Story v0174-session : la 0.17.4 — sécurité de la session : le jeton de renouvellement haché, l'avertissement avant la fermeture (#890, #895, #896, #898, #899)

Status: split

*Fiche parente, écrite le 2026-11-05 (Opus 5.5, autonomie) sur la liste acceptée par le Project Lead le même jour. Worktree
`/home/gcorbaz/devel/kesh-0174s`, branche `story/v0174-session`, partie de `origin/main` = `7bd4d5c0` (rétrospective 0.17,
après le merge de la v0.17.3) ; port **3177**, bases `kesh_v0174s` et `kesh_e2e_v0174s`. Clé courte `v0174s`, registre
`v0174-choix-autonomes-v0174s.md` (**C-v0174s-1 à 16**). Numéros de ligne de `7bd4d5c0`. Découpée (C-v0174s-1) en deux
sous-fiches sœurs, sur la même branche, développées dans cet ordre :*

- ***`v0174-session-1.md`** — le serveur : #890 (P1 sécurité), #896, les LOW serveur de #898 et #899 ;*
- ***`v0174-session-2.md`** — le navigateur : #895, les LOW navigateur de #898 et #899.*

*Issues en `refs` seulement dans les commits. Prochaine étape : validation P1 (Sonnet, lentilles R et F,
`v0174-session-validate-prompt-p1.md`), sur les trois fichiers.*

## Story

En tant qu'Administrateur dont la sauvegarde ou la base pourrait être lue par un tiers, je veux que Kesh n'y garde
aucun jeton de session utilisable ; en tant qu'utilisateur qui lit un long document dans l'aperçu, je veux être averti
avant que ma session ne se ferme, et pouvoir la garder d'un clic.

## Contrainte

Kesh **0.17.3** est en service sur le NAS du Project Lead avec ses livres réels. Essais sur bases jetables seulement
(`kesh_v0174s`, `kesh_e2e_v0174s`, migrations du worktree, seed `scripts/seed-dev-db.sql`) ; fixtures synthétiques ;
aucune donnée réelle. **Aucune migration** (C-v0174s-2) : `ls crates/kesh-db/migrations/*.sql | wc -l` = **89** avant et
après ; P2 à P8 sans objet ; **la 0.17.3 redémarre sur une base passée par la 0.17.4** (C-v0174s-7), au prix d'une
reconnexion ; une sauvegarde 0.17.3 s'importe dans la 0.17.4 (schéma identique ; ses jetons clairs sont effacés dans la
transaction d'import, C-v0174s-6).

## Ce que le code dit, par issue (`7bd4d5c0`)

### #890 — le jeton de renouvellement est stocké en clair (P1 sécurité)

**Cause, lue au code** : `crates/kesh-db/src/repositories/refresh_tokens.rs:29-33` insère `new.token` tel quel
(`.bind(&new.token)`), et toutes les recherches lient la valeur reçue du cookie : `:15-17`, `:19-20` (`WHERE token = ?`),
`:111-116` (`revoke_by_token`), `:168-171` (`set_idle_deadline`), `:191-195` (`was_rotated_within`). La valeur est un UUID v4
(`uuid::Uuid::new_v4()`, `routes/auth.rs:307` connexion, `:506` renouvellement, `:615` changement de mot de passe ;
`routes/setup.rs:292` assistant), dans une colonne `CHAR(36)` contrainte au format UUID
(`migrations/20260405000001_auth_refresh_tokens.sql:7,14-15`). Le commentaire d'entité promettait l'empreinte pour la
story 1.6 (`entities/refresh_token.rs:9-12`), jamais faite ; `crates/kesh-api/README.md:180` le reconnaît.

**Ce que voit un lecteur de la base** : la ligne active d'une session donne son cookie de renouvellement, utilisable
jusqu'à l'échéance d'inactivité (`expires_at`, 60 minutes par défaut, jusqu'à 24 h) — par `POST /api/v1/auth/refresh`
(cookie **ou corps**, `routes/auth.rs:424-427`).

**Où va la table** :

- **la sauvegarde `.keshbackup`** l'emporte (`kesh-db/src/backup.rs:114`, `TABLES_TO_TRUNCATE` ; `admin_backup/mod.rs:3-4`) ;
- **l'export de souveraineté** (ZIP) l'**exclut** déjà (`exports/global.rs:214-217`, « SECRET et éphémère ») ;
- **la restauration complète** remplace la table puis révoque toute ligne active
  (`routes/admin.rs:487-497`, `revoke_all_in_tx`, motif `password_change`, 0.17.3) ;
- **le nettoyage du démarrage** efface les lignes expirées ou révoquées depuis plus de 7 jours (`main.rs:230-245`,
  `delete_expired_and_revoked`, `refresh_tokens.rs:228-245`) ;
- **la rotation** révoque l'ancien jeton (`routes/auth.rs:500`) et crée le neuf (`:506-517`) ; la détection de vol relit un
  jeton révoqué (`:431-476`), la grâce de rotation le compare à l'horloge de la base (`:449-453`).

**Prémisse de l'issue à corriger** : « le manuel d'administration affirme un hash » **n'est plus vrai** — la 0.17.2 l'a
corrigé, comme l'issue l'annonce : `docs/manual/fr/admin-manual.tex:3027` dit « stocké en DB … **tel quel** (non haché,
suivi par \#890) ». Restent faux, ou à reformuler après le correctif : `admin-manual.tex:2263` (« ainsi que les **jetons
de session** »), `:3027` lui-même, `crates/kesh-api/README.md:162` et `:180`, `entities/refresh_token.rs:9-12`,
`admin_backup/mod.rs:3-4`. Les clés d'API, les clés MCP et les jetons de réinitialisation sont déjà hachés (SHA-256 hex,
`auth/api_key.rs:98-109`, `:121-127`) : **le jeton de renouvellement est le dernier secret de session gardé en clair.**

**Conception** : `v0174-session-1.md` § *Conception* — empreinte SHA-256 tronquée et mise en forme dans la colonne
existante (C-v0174s-2), calculée par le dépôt (C-v0174s-3), jetons d'avant invalidés et effacés (C-v0174s-6).

### #895 — avertir avant la fermeture pour inactivité (CR accepté)

**Ce que fait la 0.17.3** : `frontend/src/lib/app/stores/session-idle.svelte.ts` ferme la session à
`max(lastActivity) + N` (minuterie d'une seconde, `:283-300`), sans avis préalable (`closeSession`, `:321-351`). Les
gestes faits dans l'`<iframe>` de l'aperçu PDF ne parviennent pas à la page (C-v0172s-36) ; le manuel utilisateur demande
de sortir le pointeur du document au moins une fois par durée réglée (`user-manual.tex:97`) et renvoie à #895, comme le
manuel d'administration (`admin-manual.tex:3043`).

**Conception** : `v0174-session-2.md` — un avis une minute avant l'échéance (C-v0174s-10), « Rester connecté »
(C-v0174s-12), sans déplacement du focus (C-v0174s-11), au-dessus d'un dialogue ouvert sans le fermer (C-v0174s-13),
aucun réglage neuf.

### #896 — le paramètre ignoré de `from_fields_for_test` (dette P3)

`crates/kesh-api/src/config.rs:441-455` : `_refresh_inactivity_ignored: TimeDelta`, ignoré depuis la 0.17.3, septième
argument de **85** appels dans **80** fichiers (C-v0174s-8). Retrait mécanique : `v0174-session-1.md`.

### #898, #899 — les LOW des revues de code P1 et P2 de la 0.17.3

Triés un par un ci-dessous (C-v0174s-9).

## Triage des LOW (#898, #899)

Légende : **1** = corrigé par `v0174-session-1` ; **2** = corrigé par `v0174-session-2` ; **—** = non corrigé, motif
écrit ; **issue** = non corrigé, issue demandée (défaut réel).

| Constat | Lieu (lu à `7bd4d5c0`) | Décision | Motif ou remède |
|---|---|---|---|
| #898 S-B2 / S-E2 — `now >= deadline` → `>` survit | `middleware/auth.rs:155-157` | **1** | test `idle_exp = now` ⇒ 401 (AC-9 de la 1) ; la mutation est tuée sauf si la requête franchit la seconde (rare, déclaré) |
| #898 S-A8 — `.trunc_subsecs(3)` retiré, équivalent sous MariaDB | `routes/auth.rs:48`, `:743` | **1** | commentaire : garde contre un serveur réglé en `TIME_ROUND_FRACTIONAL`, mutation équivalente déclarée |
| #898 S-B3 / S-E4 (a) — le battement peut faire reculer l'échéance | `refresh_tokens.rs:161-181` | **—** | conception : un abaissement du réglage **doit** pouvoir raccourcir l'échéance à la prochaine activité (C-v0172s-38) ; `GREATEST` l'interdirait. Deux battements réordonnés supposent deux onglets envoyant à moins de 15 s l'un de l'autre, ce que `heartbeatAt` partagé exclut hors course de quelques ms ; l'effet est borné à l'écart (a). Documenté au doc-comment de `set_idle_deadline` (la 1) |
| #898 S-E3 — la grâce de 30 s survivrait à 5 s | `routes/auth.rs:412` ; `auth_e2e.rs` | **1** | deux cas : `revoked_at` reculé de 29 s ⇒ grâce, de 31 s ⇒ vol |
| #898 S-E4 (b) — battement parti avec l'ancien cookie après une rotation | `routes/auth.rs:703-780` | **—** | sans effet : le 401 passe par le chemin 401 du navigateur, qui renouvelle avec le cookie neuf (partagé par les onglets) et rejoue (`api-client.ts`, chemin 401) |
| #898 S-E4 (c) — un renouvellement recopie une échéance lue avant un battement | `routes/auth.rs:506-517` | **1** (texte) | perte d'au plus 15 s, rattrapée au battement suivant : l'écart (a) du manuel d'administration (`admin-manual.tex:3039`) la nomme désormais |
| #898 S-E5 — `refresh` lit N après avoir tourné le jeton ; `setup` après avoir créé l'Administrateur | `routes/auth.rs:500-529` ; `setup.rs:262-283` | **1** | lire N **avant** l'étape 8 (`refresh`) et avant la section critique (`setup`) : une panne de lecture ne laisse plus ni jeton tourné sans cookie neuf, ni Administrateur créé sous un 500 |
| #898 S-E6 — `expires_at < now` au renouvellement contre `now >= idle_exp` au middleware ; battement à `idleMs ≥ N` | `routes/auth.rs:481` ; `:734-756` | **1** / **—** | `<=` au renouvellement (alignement, mécanique) ; le battement à `idleMs ≥ N × 60 000` (échéance = maintenant, réponse `idleExpiresIn: 0`) n'est jamais envoyé par le navigateur (il ferme avant) et la requête suivante est refusée : sans portée, non corrigé |
| #898 N-B3 / N-E7 — un battement en vol à la fermeture peut écraser la redirection « inactivité » | `session-idle.svelte.ts:302-351` ; `api-client.ts` (chemin 401) | **issue** | le 401 du battement n'arrive que si l'échéance du serveur est déjà passée (écarts a, c) ; l'effet est le message affiché (« session expirée » au lieu d'« inactivité »), la session étant fermée de toute façon. Le remède touche le chemin 401 générique d'`api-client.ts` (qui écrase tout `signal` passé, `api-client.ts:304-326`) : hors périmètre minimal |
| #898 N-B5 — un renouvellement tenté à chaque chargement pour un visiteur déconnecté | `auth.svelte.ts` (`hydrate`) ; `lib.rs:1336` | **—** | vérifié : `/api/v1/auth/refresh` n'a aucun limiteur (`lib.rs:1336`, hors des `RateLimiter` de connexion et de récupération) ; un refus coûte une recherche par index. Rien à faire |
| #898 N-A3 / N-E3 — mutations plausibles non gardées (régulation d'une seconde, `generation`, `heartbeatInFlight`, `isValidMinutes`) | `session-idle.svelte.ts:61`, `:147`, `:150`, `:79` | **2** | un cas court par ligne |
| #898 N-A5, N-A6, N-A10 — procédé de la recette 0.17.3 (jeton à 1 min, réglage par SQL, captures pleine page) | recette 0.17.3 | **—** | la 0.17.3 est publiée ; pas un défaut du code. La leçon s'applique à la recette de cette story : `KESH_JWT_EXPIRY_MINUTES` ≥ N pour tout `curl` 401, réglage par l'écran, captures au viewport (AC de la 2) |
| #898 N-A8 / N-E10 — un test qui passe à vide, une assertion tautologique, des cas à `t = 0` | `session-idle.svelte.test.ts` | **2** | (9) écrit comme garde de principe, assertion littérale `470_000`, cas de `currentLast` démarrés à `1_000_000` |
| #898 N-E5 — `previousReported = -Infinity` jamais réécrit après un échec | `session-idle.svelte.ts:293`, `:315` | **2** | `removeItem` de la clé quand `previousReported` n'est pas fini |
| #898 N-E9 — `localStorage` lu à chaque `pointermove` | `session-idle.svelte.ts:217-221` | **—** | micro-coût sans mesure (lecture synchrone d'une clé, quelques dizaines par seconde au plus) ; un raccourci `memLast + w > now` ajouterait un invariant à tenir (monotonie de la clé) pour aucun effet observable |
| #898 Rec-e1 — une sortie du document au même pixel que la précédente ne compte pas | `session-idle.svelte.ts:230-237` | **2** | `lastPointer = null` quand le pointeur quitte la page pour un `<iframe>` (`pointerout` dont `relatedTarget` est nul) |
| #899 S2-B1 — trois commentaires présentent `jwt::decode` comme le chemin de production | `middleware/auth.rs:134`, `:137` ; `auth/api_key.rs:31` | **1** | commentaires corrigés (`jwt::decode_session`) |
| #899 S2-B2 — nom d'opération de `retry_on_deadlock` | `routes/session_settings.rs:94-95` | **1** | nommer la route (`session_settings::update_session_settings`), témoin du test de course aligné |
| #899 S2-B3 — `heartbeat_refuses_api_keys` n'asserte que 403 | `tests/session_idle_e2e.rs:465` | **1** | asserter aussi le code `API_KEY_MANAGEMENT_FORBIDDEN` |
| #899 S2-E2 — doc-comment d'`activity` : « 401 puis 400 » | `routes/auth.rs:685-702` | **1** | doc corrigée : le corps est jugé avant le cookie |
| #899 S2-A1 — « vingt-deux » routes rejouées, sans `update_session_settings` | `tests/audit_route_registry.rs:159` (et le second site que cite l'issue, `:274`, à relire) | **1** | recompté depuis la table du registre, corrigé |
| #899 S2-A3, N2-A2 — deux annotations fausses de la fiche 0.17.3 | `v0172-session.md`, Change Log et Dev Agent Record | **1** | annotations **datées** ajoutées (« huit heures » était juste, « ne déconnecte personne » était faux ; « quatre, dans trois boucles »), sans réécrire les entrées passées |
| #899 N2-B2 — discontinuité de `concile` (borne non finie / finie démesurée) | `session-idle.svelte.ts:195-198` | **—** | inatteignable avec ce que le serveur émet (`idleExpiresIn` est borné par `idle_expires_in`, `routes/auth.rs:57-59`) |
| #899 N2-E1 — `localStorage` refusé en cours de session dans un seul onglet | `session-idle.svelte.ts:126-137` | **issue** | défaut réel, étroit : un onglet inactif qui garde le stockage ferme la session de l'onglet qui travaille. Le remède suppose un signal entre onglets (le stockage qui fait défaut) : hors périmètre minimal |
| #899 N2-E2 — `{1,4}` → `+` dans la regex de `?minutes=` | `routes/login/+page.svelte` | **—** | mutation quasi équivalente (la borne 1440 arrête la valeur longue) : pas un défaut |
| #899 N2-A1 — capture blanche de la recette 0.17.3 | recette 0.17.3 | **—** | procédé de recette, la fermeture était établie par le journal |

**Décompte** : 26 lignes pour les 32 identifiants des deux issues (21 de #898 — huit S, douze N, Rec-e1 — et 11 de
#899 — six S2, cinq N2 ; S-E4 est réparti sur trois lignes, d'autres lignes réunissent deux ou trois identifiants) ;
10 lignes **1**, 1 ligne **1** (texte), 1 ligne **1** / **—** (S-E6), 4 lignes **2**, 8 lignes **—**, 2 lignes
**issue** (10 + 1 + 1 + 4 + 8 + 2 = 26). *Recompté depuis la table le 2026-11-05 (chaque identifiant cherché dans la
table : aucun absent).*

## Issues demandées à l'orchestrateur (« on trace toujours »)

1. **N-B3 / N-E7** (P4) — « Session : un battement en vol à la fermeture pour inactivité peut afficher “session expirée”
   au lieu du message d'inactivité » ; lieu et motif ci-dessus.
2. **N2-E1** (P3) — « Session : `localStorage` refusé en cours de session dans un seul onglet, un onglet inactif ferme la
   session d'un onglet actif » ; lieu et motif ci-dessus.
3. **Colonne `refresh_tokens.token`** (P4, dette) — « renommer la colonne en `token_digest` et corriger son commentaire
   `'UUID v4 opaque'` à la prochaine migration de la table » (C-v0174s-2).
4. Si l'étape 0 de la `v0174-session-2` ne trouve pas de moyen simple pour l'avis au-dessus d'un dialogue
   (C-v0174s-13) : l'écart, en issue.

À la fermeture de #898 et #899 par la PR, l'orchestrateur y laisse un commentaire qui renvoie à la table ci-dessus (ce
qui est corrigé, ce qui ne l'est pas et pourquoi).

## Modules (règle de splitting)

| Sous-fiche | Modules | Nombre |
|---|---|---|
| `v0174-session-1` | `crates/kesh-db` (dépôt `refresh_tokens`, entité, `Cargo.toml`, tests du dépôt) ; `crates/kesh-api` (commentaires et ordre de lecture dans `routes/auth.rs`, `routes/setup.rs`, `routes/admin.rs`, `routes/session_settings.rs`, `middleware/auth.rs`, `auth/api_key.rs`, `admin_backup/mod.rs`, `config.rs`, README, tests) ; textes | 2 |
| `v0174-session-2` | `crates/kesh-i18n` (quatre catalogues) ; `frontend/src/lib/app/stores` (`session-idle`) ; `frontend/src/lib/shared/components` (avis neuf) ; `frontend/src/routes` (layout racine) ; textes | 4 |

Chaque sous-fiche reste sous le seuil de cinq ; aucune ne dépend du code de l'autre (C-v0174s-1). La `v0174-session-1`
touche un dépôt (`repositories/refresh_tokens.rs`) : **gate complet même en cours de boucle** (`CLAUDE.md`, exception
`kesh-db`).

## Textes : qui écrit quoi

- **Manuel d'administration** (`docs/manual/fr/admin-manual.tex` et PDF) : la 1 écrit § *Authentification JWT + refresh
  tokens* (`:3021-3030`), § sauvegarde (`:2263`), l'écart (a) (`:3039`) et une phrase « Mise à jour vers la 0.17.4 » ;
  la 2 écrit § *Durée d'inactivité* (`:3043`, « Ce qui compte » ; `:3047`, `localStorage`). La 2 régénère le PDF en
  dernier (`make fr` dans `docs/manual/`, sous les deux verrous).
- **Manuel utilisateur** (`user-manual.tex` et PDF) : la 2 seule (`:95-97`).
- **CHANGELOG** : section `## [0.17.4] — Non publié` en tête ; la 1 écrit *Sécurité* (#890) et *Corrigé* (LOW serveur,
  #896 n'y figure pas : sans effet pour l'utilisateur) ; la 2 écrit *Ajouté* (#895) et *Corrigé* (LOW navigateur). Trois
  stories de la 0.17.4 y écrivent : union à l'intégration.
- **`crates/kesh-api/README.md`** : la 1.
- **Aucune macro de version** des manuels n'est touchée (gate de release, hors story).

## Change Log

- 2026-11-05 — Fiche créée (Opus 5.5, autonomie) : découpage en deux sous-fiches (C-v0174s-1), empreinte sans migration
  (C-v0174s-2 à 7), retrait mécanique de #896 (C-v0174s-8), triage des LOW (C-v0174s-9), avertissement (C-v0174s-10 à
  15), registre (C-v0174s-16). Prémisse corrigée : le manuel d'administration ne prétend plus le hachage depuis la 0.17.2.
  Prochaine : validation P1 (Sonnet, R et F).
