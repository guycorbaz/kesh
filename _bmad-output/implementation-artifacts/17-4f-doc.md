# Story 17.4f: Documentation recovery (manuels + .env.example + CHANGELOG/README)

Status: review

<!-- Extraite de la spec parente UMBRELLA 17-4 (validate CONVERGÉ 6 passes), Partie F : AC25-28. Re-validate optionnel. -->
<!-- DERNIÈRE sous-story de l'épopée 17-4 (a-e DONE) — sa complétion FERME l'issue #122 et l'épopée. Doc-only : la règle Test Locally First ne s'applique pas (CI no-op), MAIS le build LaTeX doit passer. -->

## Story

As a **opérateur (DevOps NAS) ou utilisateur final de Kesh**,
I want **une documentation complète du recovery par email — section SMTP du manuel admin (8 variables + recommandations + limitations héritées), workflow « mot de passe oublié » du manuel user (en corrigeant le TTL erroné existant), section `.env.example`, entrée CHANGELOG, README, sémantique PUT dans la doc API externe et recette de test**,
so that **un déploiement puisse activer le recovery sans lire le code, que les limitations connues soient assumées par écrit, et que l'épopée 17-4 (#122) soit fermée proprement**.

## Contexte & cadrage

**Issue source :** [#122](https://github.com/guycorbaz/kesh/issues/122) — **cette story la FERME** (`closes #122` au commit final). Épopée 17-4 : a✅ b✅ c✅ d✅ e✅ → **f (ici, dernière)**.

**Ce qui doit être documenté (livré par a-e) :** feature opt-in `KESH_FEATURE_FORGOT_PASSWORD` (défaut false) ; 8 vars SMTP/recovery avec fail-fast boot si incomplètes ; flux : lien « Mot de passe oublié ? » sur login (conditionnel) → `/forgot-password` (message générique anti-énum) → email avec lien `{KESH_PUBLIC_BASE_URL}/reset-password?token=` (**TTL 30 min**, usage unique) → nouveau mot de passe (≥ 12) → toutes les sessions déconnectées ; compte sans email = non-recouvrable par ce flux → fallback break-glass `KESH_ADMIN_RESET` (inchangé, manuel admin §6) ; rate-limit 5 req/15 min/IP partagé (blocage 30 min).

**Dettes héritées dont 17-4f est propriétaire :**
- **ECH2-2 (17-4a)** : sémantique REMPLACEMENT du `PUT /users/:id` (email absent du corps ⇒ effacé) → doc API externe (le frontend renvoie toujours l'email depuis 17-4d ; un client PAT doit le savoir).
- **D3 (17-4c)** : exigence horloge NTP/UTC (skew app vs MariaDB fatal sur un TTL de 30 min) → manuel admin.
- **L-C1/#173 (17-4c)** : rate-limit par IP inopérant derrière reverse proxy (IP partagée → 5 req bloquent tout le monde 30 min) → manuel admin, renvoi #173.
- **L-C2/L-C3 (17-4c)** : blocage 30 min = TTL ; envoi détaché non drainé au shutdown → manuel admin §Limitations.
- **Dette de validation AC24 (umbrella)** : test manuel de réception email réelle multi-providers (hors-CI) → procédure documentée.

**Scope :** T-F1 manuel admin (+PDF si convention) ; T-F2 manuel user (+PDF) **+ correction du TTL erroné « 1 heure » existant** ; T-F3 `.env.example` ; T-F4 CHANGELOG + README ; T-F5 doc API externe (ECH2-2) + docs/testing.md (recette E2E feature-on) ; T-F6 gate doc.

**Hors scope :** site web `website/` (sera synchronisé au moment de la release v0.2.0, règle pré-release CLAUDE.md — la feature n'est pas encore sur `main`) ; manuels DE/IT/EN (v0.2+ traductions) ; toute modification de code.

## Décisions de conception

- **DF-1 — placement admin** : nouvelle sous-section dans la §5 Configuration (après le dernier tableau de vars, ~l.780, AVANT la §6 break-glass) : « Récupération de mot de passe par email (SMTP) » — tableau des 8 vars, recommandations (TLS=true sinon le token transite en clair, FROM valide, base URL sans slash final), renvoi croisé §6 (fallback). Les limitations (D3 NTP/UTC, L-C1 proxy/#173, L-C2, L-C3) dans cette même sous-section ou en §Sécurité selon le style du fichier — figer au dev en lisant les sections voisines.
- **DF-2 — user manual** : enrichir la sous-section existante « Récupération de mot de passe » (§2, l.81-87) — **CORRIGER « valide 1 heure » → « valable 30 minutes »** (bug doc pré-existant), décrire les 2 pages, le message générique (on ne confirme jamais l'existence d'un compte), l'usage unique, la déconnexion des autres sessions, et « si vous ne recevez rien : vérifiez vos indésirables / votre compte n'a peut-être pas d'email → contactez votre administrateur ». Renvoi croisé §5 changement de mot de passe.
- **DF-3 — .env.example** : nouvelle section calquée sur le format existant (`<EDIT: …>` / commentaires multilignes / note « si non configuré, recovery = break-glass KESH_ADMIN_RESET »), placée près des sections auth/admin. ⚠️ Warning explicite sur `KESH_SMTP_PASSWORD` (secret) et `KESH_SMTP_TLS=true` recommandé.
- **DF-4 — CHANGELOG** : entrée sous `[Non publié] → Added` (le bloc existe), style Keep a Changelog, réf #122, renvois manuels. Ne PAS renommer en 0.2.0 (ça se fait à la release).
- **DF-5 — PDFs** : la convention projet versionne les PDFs (CLAUDE.md, précédent PR #102) mais l'exploration suggère qu'ils ne sont plus trackés — **vérifier ground-truth au dev** (`git ls-files docs/manual/**/*.pdf`) : si trackés → régénérer (`make` dans docs/manual, xelatex) et commiter ; sinon → build de vérification seulement (le .tex doit compiler) sans commiter les PDFs.
- **DF-6 — api-external.md** : courte section « Endpoints utilisateurs — sémantique du PUT » (ECH2-2 : remplacement intégral, toujours renvoyer `email`) + note « les endpoints publics de recovery ne sont PAS des endpoints PAT ; une clé PAT ne peut pas déclencher un reset pour un autre utilisateur ».

## Acceptance Criteria

> Numérotation umbrella (AC25-28, Partie F) + extensions héritées.

25. **Manuel admin FR** : sous-section « Récupération de mot de passe par email (SMTP) » dans §Configuration — tableau des 8 vars (`KESH_FEATURE_FORGOT_PASSWORD`, `KESH_SMTP_HOST/PORT/USER/PASSWORD/FROM/TLS`, `KESH_PUBLIC_BASE_URL`) avec défauts et obligation conditionnelle (fail-fast) ; exemples de providers (Gmail/app-password, Postfix local, Synology Mail Server) ; recommandation TLS ; **limitations documentées** : D3 NTP/UTC (TTL 30 min sensible au skew), L-C1 proxy/#173, L-C2 blocage=TTL, L-C3 shutdown ; renvoi croisé au break-glass §6 ; **procédure de test manuel multi-providers** (dette AC24 : envoyer un vrai email de test vers ≥ 1 provider après config, hors-CI). Annexe liste des vars synchronisée si elle existe (§15).
26. **Manuel user FR** : sous-section « Récupération de mot de passe » enrichie ET corrigée (TTL **30 minutes**, pas « 1 heure ») : workflow complet utilisateur, anti-énumération expliquée simplement, usage unique, sessions déconnectées, cas « pas d'email reçu ».
27. **`.env.example`** : section recovery/SMTP complète (8 vars commentées, format maison `<EDIT:>`/`<GENERATE_ME:>`), warning secret + TLS, note fallback break-glass.
28. **CHANGELOG `[Non publié]/Added`** (réf #122, renvois manuels) + **README** : bullet « Récupération de mot de passe par email » dans Fonctionnalités + feuille de route Epic 17 cohérente (recovery livré côté branche — formulation au présent de la branche, le statut global v0.2 reste 🚧).

### Étendus (dettes héritées)

- **AC-F5** : `docs/api-external.md` — sémantique PUT remplacement (ECH2-2) + note endpoints publics ≠ PAT.
- **AC-F6** : `docs/testing.md` — recette backend E2E feature-on (env vars exactes validées en 17-4e : `KESH_SMTP_USER`, `KESH_ADMIN_PASSWORD` ≥ 12, port custom si 80/8080 occupés) + mention des 14+5 tests et de l'endpoint d'injection test-mode.
- **Transverse** : si les PDFs sont versionnés (DF-5), les régénérer et les commiter dans le MÊME commit ; cohérence des renvois croisés ; aucune modification de code.

## Tasks / Subtasks

- [x] **T-F1** Manuel admin : sous-section SMTP/recovery + limitations héritées + procédure test manuel + annexe vars sync. (AC: 25)
- [x] **T-F2** Manuel user : enrichir + **corriger TTL 1h→30min**. (AC: 26)
- [x] **T-F3** `.env.example` section recovery. (AC: 27)
- [x] **T-F4** CHANGELOG `[Non publié]` + README (Fonctionnalités + feuille de route). (AC: 28)
- [x] **T-F5** `docs/api-external.md` (ECH2-2 + note PAT) + `docs/testing.md` (recette feature-on). (AC: F5, F6)
- [x] **T-F6** Gate doc : build LaTeX des 2 manuels FR sans erreur (`make` ou xelatex direct, DF-5 pour le commit des PDFs) ; relecture des renvois croisés ; commit `closes #122`.

## Dev Notes

### Ground-truth doc (exploration 2026-06-12, 40 lectures)

- **admin-manual.tex (1794 l.)** : §5 Configuration l.637-780 (tableaux de vars — suivre leur format exact), §6 break-glass l.843-887 (la nouvelle sous-section la précède et y renvoie), §10 Sécurité l.1329, §15 Annexes l.1711 (liste vars à synchroniser si présente). Style versionné via `shared/kesh-style.sty` (`\keshVersion`).
- **user-manual.tex (1031 l.)** : §2.4 « Récupération de mot de passe » l.81-87 — 3 lignes sommaires avec le **TTL FAUX (« 1 heure »)** ; §5 Gestion du compte l.180 (changement mdp l.193, renvoi croisé).
- **Build** : `docs/manual/README.md` l.16 — Makefile cibles `make admin` / `make user` (xelatex). **DF-5** : trancher le commit des PDFs par `git ls-files 'docs/manual/**/*.pdf'`.
- **.env.example (204 l.)** : 14 sections, format `<EDIT:>`/`<GENERATE_ME:>` + références Story/issue dans les commentaires ; insérer la section recovery après les sections auth (~l.137-176, AVANT KESH_TEST_MODE). Les 8 vars n'y figurent PAS encore.
- **CHANGELOG.md (260 l.)** : Keep a Changelog FR, `[Non publié]` l.11 avec Added/Fixed existants ; dernière release 0.1.8 (2026-06-04).
- **README.md (195 l.)** : Fonctionnalités l.23 (pas de bullet recovery), Feuille de route l.165 (v0.2 🚧 Epic 17 : PAT ✓, export/import ✓, recovery à marquer livré-branche).
- **api-external.md (~400 l., 17-2c)** : §1-8 ; pas de section users dédiée — ajouter une courte section sémantique PUT (ECH2-2) + note recovery ≠ PAT.
- **testing.md (143 l.)** : §4 prérequis Playwright (2 terminaux) — y greffer la recette feature-on (header de `password-recovery.spec.ts` comme source : `KESH_SMTP_USER` pas USERNAME, `KESH_ADMIN_PASSWORD` ≥ 12, `KESH_PORT` custom, valeurs SMTP factices OK car token injecté via `/_test/password-reset-token`).
- **Valeurs exactes à documenter (vérifiées 17-4b/c/e)** : défauts `KESH_SMTP_PORT=587`, `KESH_SMTP_TLS=true` ; TTL 30 min (`PASSWORD_RESET_TTL_MINUTES`) ; rate-limit 5/15 min/30 min ; min mdp 12 (`KESH_PASSWORD_MIN_LENGTH`) ; message d'erreur boot fail-fast liste les vars : `KESH_SMTP_HOST, KESH_SMTP_USER, KESH_SMTP_PASSWORD, KESH_SMTP_FROM, KESH_PUBLIC_BASE_URL`.

### Pièges

- Ne pas inventer de vars (`KESH_SMTP_USERNAME` n'existe pas — c'est `KESH_SMTP_USER`, vérifié sur pièces en 17-4e).
- Le README ne doit pas OVER-promettre : la feature est sur branche, pas sur `main` — le tableau roadmap garde v0.2 🚧 ; le bullet Fonctionnalités peut être ajouté car le README est lu depuis la branche/PR et sera mergé avec le code (cohérent règle « même commit que le code qui le motive »).
- LaTeX : échapper `_` dans les noms de vars (`\_`) selon l'usage du fichier ; suivre le style des tableaux existants.
- CHANGELOG : ne pas créer de section 0.2.0 (release séparée).

### References

- [Source: umbrella `17-4-recovery-mot-de-passe.md` — AC25-28 Partie F + transverses 29 (e : doc TLS)]
- [Source: 17-4a (ECH2-2 owner f), 17-4c (D3, L-C1/#173, L-C2/3), 17-4e (recette DE-6 corrigée, endpoint injection) — story files]
- [Source: docs/manual/fr/admin-manual.tex:637-887,1329,1711 ; user-manual.tex:81-87,180-203 ; docs/manual/README.md:16]
- [Source: .env.example (14 sections, format placeholders) ; CHANGELOG.md:11 ; README.md:23,165]
- [Source: docs/api-external.md (17-2c) ; docs/testing.md:§4]
- [Source: CLAUDE.md §Synchroniser TOUTES les docs (manuels versionnés PDF cf. PR #102 — à re-vérifier DF-5), §Quand sauter (doc-only)]

## Dev Agent Record

### Agent Model Used

Claude Fable 5 (dev-story single-pass, 2026-06-12).

### Debug Log References

### Completion Notes List

- **DF-5 tranché ground-truth** : `git ls-files 'docs/manual/**/*.pdf'` liste les 3 PDFs → versionnés (CLAUDE.md/PR #102 avait raison, l'exploration s'était trompée) → régénérés via `make admin user` (xelatex, 0 erreur) et commités.
- **T-F1** : sous-section `\label{sec:recovery-smtp}` insérée dans §Configuration AVANT le break-glass — tableau 8 vars (format tabularx du fichier), fail-fast, warning TLS, 3 exemples providers, garanties (anti-énum/usage unique/révocation/audit), procédure de test manuel multi-providers (dette AC24), 4 limitations héritées (D3 NTP/UTC avec note docker-compose horloge partagée, L-C1 #173, L-C2, L-C3), keshtip complémentarité break-glass. Annexe §15 = simple renvoi `\ref{sec:env-vars}` → rien à synchroniser.
- **T-F2** : §2.4 user réécrite — **TTL corrigé « 1 heure » → 30 minutes** (bug doc pré-existant), workflow 5 étapes, anti-énum vulgarisée, keshwarning enrichi (cas sans email), keshtip (lien expiré/rate-limit/renvoi changement de mdp).
- **T-F4** : la feuille de route README listait déjà « recovery mot de passe » dans E17 🚧 — rien à changer (pas d'over-promise) ; bullet Fonctionnalités ajouté.
- **T-F5** : `api-external.md` §8 bis (sémantique REMPLACEMENT du PUT, bonne pratique GET-puis-PUT-complet, note recovery ≠ PAT avec pointeur `PUT /users/:id/reset-password` admin) ; `testing.md` recette feature-on complète avec les pièges vérifiés 17-4e.
- Doc-only : aucun code touché, CI no-op attendue ; gate = build xelatex 2 manuels OK.

### File List

**Modifiés (aucun nouveau fichier) :**
- docs/manual/fr/admin-manual.tex — sous-section « Récupération de mot de passe par email (SMTP) » (~95 lignes)
- docs/manual/fr/admin-manual.pdf — régénéré
- docs/manual/fr/user-manual.tex — §2.4 réécrite (TTL corrigé)
- docs/manual/fr/user-manual.pdf — régénéré
- .env.example — section recovery/SMTP (8 vars commentées)
- CHANGELOG.md — entrée [Non publié]/Added #122
- README.md — bullet Fonctionnalités
- docs/api-external.md — §8 bis sémantique PUT (dette ECH2-2) + note recovery ≠ PAT
- docs/testing.md — recette backend E2E feature-on (17-4e)
- _bmad-output/implementation-artifacts/{17-4f-doc.md,sprint-status.yaml}

## Change Log

### Dev-story (Fable 5, 2026-06-12)

- T-F1..T-F6 single-pass, doc-only (AC25-28 + AC-F5/F6). Dettes héritées soldées par documentation : ECH2-2 (PUT remplacement, api-external), D3 (NTP/UTC manuel admin), L-C1/#173, L-C2, L-C3, dette de validation AC24 (procédure test manuel).
- Bug doc pré-existant corrigé : TTL « 1 heure » → 30 minutes (manuel user §2.4).
- PDFs versionnés régénérés (DF-5 ground-truth). Build xelatex 0 erreur.
- Cette story ferme #122 (le `closes` est dans le message du commit dev ; à reporter aussi dans la description de la PR umbrella pour le squash).
