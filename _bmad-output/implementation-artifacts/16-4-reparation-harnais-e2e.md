# Story 16.4 : Réparation du harnais E2E — et de quoi voir sa prochaine panne

## Status

draft

## Story

**As a** mainteneur de Kesh,
**I want** que la suite Playwright s'exécute réellement, et qu'une suite qui cesse de s'exécuter **se signale**,
**so that** les stories qui exigent une preuve de bout en bout puissent la produire — et qu'on ne redécouvre pas la panne des mois plus tard, à l'occasion d'une story sans rapport.

Issue : **#285**. Story de l'Epic 16, **traitée dans l'epic en cours** parce qu'elle bloque une story active (§ *Tech debt management*, triage hors fenêtre rétrospective : « critique pour l'Epic en cours → story dans l'Epic N+1 en cours, traiter immédiatement »).

⚠️ **Bloque la livraison de 16-2b**, dont l'`AC-B8` prescrit quatre exécutions Playwright et dont **trois mutations sur quatre** s'appuient dessus. Ses tests sont écrits ; ils n'ont jamais pu tourner.

---

## Contexte — ce qui est établi, et comment

`authedApiContext()` (`frontend/tests/e2e/helpers/test-state.ts:174`) rend un `APIRequestContext` **non authentifié** : tout appel API qu'il émet reçoit **401**. La quasi-totalité des specs montant leurs fixtures par ce helper, **la suite entière est inopérante** — pas une spec en particulier.

**Le diagnostic est fait, il n'est pas à refaire.** Le login pose des cookies `SameSite=Strict` :

```
set-cookie: kesh_access_token=…; HttpOnly; SameSite=Strict; Path=/; Max-Age=900
set-cookie: kesh_refresh_token=…; HttpOnly; SameSite=Strict; Path=/api/v1/auth; Max-Age=2592000
```

Le helper lit le `storageState` du navigateur — **les cookies y sont**, le garde-fou `storageState.cookies.length === 0` (`:181`) n'ayant pas levé — puis crée un **contexte API séparé**. Une requête émise depuis un contexte API n'a **pas de site initiateur** : un cookie `SameSite=Strict` n'y est pas joint. Le navigateur est authentifié, le contexte API ne l'est jamais.

**Cinq hypothèses écartées, chacune par une vérification** *(2026-08-04, cf. #285)* :

| Hypothèse | Vérification | Verdict |
|---|---|---|
| Bac à sable réseau de l'outillage | backend et Playwright lancés hors sandbox | écartée |
| Backend ou auth cassés | `curl` : login **200**, puis `POST /api/v1/accounts` avec le cookie → **201**, compte créé | écartée |
| Cookie `Secure` sur du HTTP | `KESH_COOKIE_SECURE=false`, l'en-tête ne porte pas `Secure` | écartée |
| Login de la page en échec | log `login success user_id=1`, et le garde-fou du helper n'a pas levé | écartée |
| Régression d'une story récente | `invoice-revenue-account.spec.ts` (Story 16-1b) échoue à l'identique | écartée |

**La date de la régression n'est PAS établie.** Le `SameSite=Strict` remonte au passage des tokens en cookies HttpOnly (Story 10-5), mais rien ne prouve que le harnais soit cassé depuis lors. Ne pas l'affirmer dans le CHANGELOG.

---

## Le vrai sujet : pourquoi personne ne l'a vu

Trois circonstances se cumulent, et **la troisième est celle que cette story doit fermer** :

1. **La CI n'exécute pas les E2E** — `.github/workflows/ci.yml` ne lance que `Backend (Rust)`, `Frontend (Svelte)` et `Docker build (sanity)`. C'est documenté et assumé au `CLAUDE.md`.
2. **Les tests manuels du mainteneur passent par un navigateur**, sur le NAS, jamais par Playwright *(confirmé par Guy, 2026-08-04)*.
3. **Rien ne signale une suite qui ne s'exécute pas.** Un test jamais lancé ne rougit pas — **il se tait.**

C'est le mode d'échec du **test muet**, celui qui a coûté `backfill_skips_archived_accounts` en 16-1a et qui s'est reproduit pendant l'implémentation de 16-2a, où quatre tests supprimés par mégarde ont rendu « 16 passed, 0 failed ». **Réparer le harnais sans traiter (3) laisse la porte ouverte à la récidive**, et la prochaine panne se découvrira de la même façon : par accident, des mois plus tard.

---

## Décisions

- **D1 — Corriger le HELPER, ne pas toucher au produit.** La correction joint explicitement les cookies d'authentification au contexte API (en-tête `Cookie` construit depuis le `storageState`, ou équivalent), dans `authedApiContext()`.

  ⚠️ **L'option « passer `SameSite` à `Lax` quand `KESH_TEST_MODE=1` » est ÉCARTÉE.** Elle modifierait une mesure de sécurité **du produit** pour arranger l'outillage de test, et ferait diverger le comportement testé de celui livré — un test qui n'exerce plus la configuration réelle ne prouve plus ce qu'il prétend. `SameSite=Strict` est un choix délibéré : il reste.

  L'option « monter les fixtures par l'interface plutôt que par l'API » est également écartée : elle rendrait la suite bien plus lente et fragile, pour un problème qui est celui du helper.

- **D2 — La réparation n'est acquise QUE si un test échoue avant et passe après.** Le critère d'acceptation n'est pas « le helper compile » ni « une spec passe », mais la **suite complète exécutée**, avec son décompte. Un helper corrigé dont on n'aurait relancé qu'une spec ne prouverait rien de la suite.

- **D3 — Deux niveaux : la suite complète EN LOCAL avant push, un SMOKE en CI.** *(Arbitrage de Guy, 2026-08-04.)*

  1. **Suite complète, en local, avant tout `git push`** — au même titre que les gates backend et frontend de la § *Test Locally First*. Elle n'est **pas** exécutée systématiquement pendant l'itération : la doctrine du dépôt reste le gate ciblé entre les passes, le gate complet au push.
  2. **Un SMOKE E2E en CI** : **une seule** spec — login, puis **un** appel API authentifié. Deux à trois minutes.

  ⚠️ **Le smoke ne teste AUCUNE fonctionnalité, et c'est délibéré.** Il vérifie que **le harnais est vivant**. C'est exactement le point où la panne de #285 se manifeste : le login réussit, l'appel API suivant rend 401. Un smoke de cette forme l'aurait attrapée le jour même.

  **Pourquoi les deux, et pas seulement le premier.** Une règle « les E2E tournent en local avant push » repose sur la discipline de qui pousse — or le `CLAUDE.md` porte déjà, sur ce point précis, la réserve « ne JAMAIS écrire *gate vert* pour un run qui n'a pas tourné ». Elle existe parce qu'on peut l'affirmer sans l'avoir fait. Et une story qui ne touche pas le frontend fait légitimement sauter les E2E : si le harnais recasse à ce moment-là, **personne ne le saura**. Le smoke est ce qui rend la panne visible sans imposer le coût de la suite entière.

  **Pourquoi pas toute la suite en CI.** MariaDB, seed, navigateurs, durée : le coût est réel et récurrent, pour une couverture que le gate local avant push assure déjà.

  ⚠️ **Sans D3, cette story répare une panne et laisse en place ce qui l'a rendue invisible.**

- **D4 — Le montage local doit être ÉCRIT.** Il n'existe aujourd'hui aucune recette documentée pour lancer les E2E : le montage employé au diagnostic a été reconstitué par tâtonnement (variables d'environnement du backend, base `kesh_e2e`, `KESH_TEST_MODE`, `KESH_STATIC_DIR`, port, `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE`). Une partie du problème est là : **ce qu'on ne sait pas lancer, on ne lance pas.**

---

## Acceptance Criteria

- **AC-1 — Le helper rend un contexte authentifié.** `authedApiContext()` produit un `APIRequestContext` dont les appels API aboutissent. Correction **dans le helper**, aucun changement du produit (D1).

- **AC-2 — La réparation est PROUVÉE par un avant/après.** Consigner, dans le Dev Agent Record, la sortie de la suite **avant** correction (échecs) et **après** (verdict), sur la **même** commande. ⚠️ **Ne pas se contenter d'une spec** : c'est la suite complète qui était inopérante (D2).

- **AC-3 — La suite complète s'exécute, et son décompte est consigné.** Nombre de specs exécutées, passées, échouées, ignorées. ⚠️ **Un `.spec.ts` ignoré parce que mal nommé ne rougit pas** — la convention du dépôt veut `*.spec.ts`, un `*.test.ts` dans `tests/e2e/` est silencieusement écarté. Vérifier que le décompte correspond au nombre de fichiers réellement présents.

  ⚠️ **Des échecs SANS RAPPORT avec le cookie sont attendus** : la suite n'a pas tourné depuis longtemps, d'autres dérives ont pu s'accumuler. Les **trier** : ce qui relève du cookie (fermé par cette story), ce qui relève d'autre chose (à tracer en issue, pas à absorber ici).

- **AC-4 — La suite devient observable, aux DEUX niveaux de D3.**
  - **Smoke en CI** : un job dédié, **une** spec (login + un appel API authentifié), ajouté à `.github/workflows/ci.yml`. Il doit rester **court** — son intérêt tient à ce qu'il coûte peu ; l'élargir le ferait dériver vers la suite complète que D3 écarte de la CI.
  - **Gate local avant push** : la suite complète, écrite comme obligation au `CLAUDE.md` § *Test Locally First* (AC-5).

  ⚠️ **Le smoke est ÉPROUVÉ** : on le met en défaut volontairement — en rétablissant le défaut d'origine du helper — et l'on constate **qu'il rougit**. Un garde-fou qu'on n'a pas vu parler n'est pas un garde-fou ; c'est toute la leçon de #285.

- **AC-5 — Le montage local est documenté au `CLAUDE.md`** (D4), § *Test Locally First* : commande complète, variables d'environnement, base de données, pré-requis (MariaDB, seed, navigateurs, `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` sur Ubuntu 26.04+), **et l'obligation de lancer la suite avant tout push**. C'est là qu'on relit les règles du dépôt — une recette rangée ailleurs ne serait pas retrouvée.

- **AC-6 — Discrimination prouvée par mutation.** Rétablir le défaut d'origine dans le helper (retirer la transmission du cookie) → **la suite rougit**. Consigné avec sa sortie, fichier restauré à l'identique ensuite. ⚠️ **Sans cette mutation, rien ne distingue « la suite passe » de « la suite ne teste plus rien ».**

- **AC-7 — Gate.** Frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) **et** la suite E2E. Verdict **lu dans le log**, état final.

---

## Tasks / Subtasks

- [ ] **T-1 — Reproduire, puis corriger** (AC-1, AC-2)
  - [ ] Monter l'environnement local et **capturer la sortie AVANT** correction.
  - [ ] Corriger `authedApiContext()` (`test-state.ts:174`) — cookies joints explicitement, produit non modifié.
  - [ ] Recapturer la sortie **APRÈS**, même commande.
- [ ] **T-2 — Exécuter la suite entière et TRIER** (AC-3)
  - [ ] Décompte : fichiers présents, specs exécutées, passées, échouées, ignorées.
  - [ ] Trier les échecs résiduels : cookie / autre chose. Ouvrir une issue pour chaque cause distincte, ne rien absorber en silence.
- [ ] **T-3 — Rendre la suite observable** (AC-4)
  - [ ] Job **smoke** dans `.github/workflows/ci.yml` : une spec, login + un appel API authentifié. Le garder court.
  - [ ] **L'éprouver** : rétablir le défaut du helper, constater que le smoke rougit, restaurer.
- [ ] **T-4 — Documenter le montage** (AC-5)
- [ ] **T-5 — Mutation** (AC-6) — retirer la transmission du cookie, constater que la suite rougit, restaurer.
- [ ] **T-6 — Gate** (AC-7) — frontend + E2E, état final, verdict lu dans le log.

---

## Dev Notes

### Ce que cette story ne doit PAS faire

- **Ne pas modifier `SameSite`, ni aucun réglage de cookie du produit** (D1). Le problème est dans le helper de test.
- **Ne pas réécrire les specs** pour contourner le contexte API (D1).
- **Ne pas absorber les échecs sans rapport avec le cookie** : les tracer en issue (§ *Issue Tracking Rule*).
- **Ne pas dater la régression** sans preuve. Le `SameSite=Strict` remonte à la Story 10-5, ce qui n'établit pas que le harnais soit cassé depuis.

### Montage employé au diagnostic — point de départ, pas référence

```sh
KESH_TEST_MODE=1 KESH_COOKIE_SECURE=false \
DATABASE_URL='mysql://kesh:kesh_dev@127.0.0.1:3306/kesh_e2e' \
KESH_JWT_SECRET='<32+ octets>' \
KESH_ADMIN_USERNAME=changeme KESH_ADMIN_PASSWORD='<12+ caractères>' \
KESH_PORT=3000 KESH_HOST=127.0.0.1 KESH_STATIC_DIR=frontend/build \
./target/release/kesh-api

cd frontend
PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 \
KESH_BACKEND_URL=http://127.0.0.1:3000 \
npx playwright test
```

⚠️ **`KESH_ADMIN_PASSWORD` exige 12 caractères minimum**, alors que les specs se connectent avec `admin`/`admin123` (8). Ces deux valeurs n'ont rien à voir : la variable ne sert qu'au **bootstrap** de l'administrateur, l'utilisateur `admin` de la base `kesh_e2e` préexiste. Le piège coûte une lecture de code.

⚠️ **`KESH_COOKIE_SECURE=false`** a été posé au diagnostic pour écarter l'hypothèse `Secure`. Vérifier s'il reste nécessaire une fois le helper corrigé.

### References

- Issue **#285** — le rapport complet, avec les cinq hypothèses écartées.
- Story **16-2b** — bloquée par celle-ci ; quatre exécutions Playwright, trois mutations qui en dépendent.
- `frontend/tests/e2e/helpers/test-state.ts` — `authedApiContext:174`, garde-fou `:181`, `resolveBackendUrl:55`.
- `CLAUDE.md` § *Test Locally First* (E2E), § *Issue Tracking Rule*, § *Tech debt management* (triage hors rétrospective).

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

**2026-08-04 — Story créée** à la demande de Guy, après le diagnostic conduit pendant l'implémentation de 16-2b et tracé en issue **#285**.

**2026-08-04 — D3 tranchée par Guy** : suite complète **en local avant push**, plus un **smoke** en CI (une spec, login + un appel API authentifié, 2-3 min). Toute la suite en CI est écartée pour son coût récurrent ; le smoke seul ne suffirait pas à couvrir les fonctionnalités, mais ce n'est pas son rôle — il vérifie que **le harnais est vivant**, ce qui est précisément le point où #285 se manifeste. AC-4 et AC-5 amendés en conséquence.

**Statut : `draft`, NON VALIDÉE.** Une boucle `bmad-create-story validate` reste à lancer. Le point qui y appelle le plus d'attention : **AC-3 anticipe des échecs résiduels sans pouvoir les nommer** — la suite n'ayant pas tourné depuis longtemps, on ignore ce qu'elle contient d'autre que le défaut du cookie.
