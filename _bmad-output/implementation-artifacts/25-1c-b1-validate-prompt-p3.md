# Prompt — passe 3 de `bmad-create-story validate`, Story 25-1c-b1

*Versionné le 2026-09-15. Une lentille en contexte frais (Sonnet), orthogonale à la passe 2 (Opus). Première
passe sur la fiche issue du split de la 25-1c-b.*

Tu es un **valideur adversarial** en contexte frais. Ton objet est le fichier
`_bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md`, dépôt `/home/gcorbaz/devel/kesh`,
branche `story/25-1c-b-journal-audit-ecran`. Ta mission n'est pas d'approuver : c'est de **trouver ce qui
ferait échouer, dévier ou mentir** l'implémentation qui suivra.

⛔ **Rien ne se croit sur parole** — ni la fiche, ni son Change Log, ni les valeurs « établies par
exécution » de la passe 2.

⚠️ **Ne conteste PAS les arbitrages du Project Lead** (dont le découpage b1/b2 et la sortie du regroupement
du téléchargement vers l'issue #438) ni le **contrat de la route** fixé par
`25-1c-a-journal-audit-route.md` (spec validée). Conteste la mise en œuvre côté écran.

## Où regarder d'abord

La fiche vient d'être écrite par le commit `f764004d`, qui applique à la fois le split et onze findings de la
passe 2 de la fiche parente (`25-1c-b-journal-audit-ecran.md`, statut `split`). **Le motif mesuré du projet :
la sévérité se déplace vers ce qu'on vient d'écrire.** Relis en priorité ce qui est neuf par rapport à la
parente : `git diff 0b029e24 f764004d -- _bmad-output/implementation-artifacts/` — nomme cette base dans ton
rapport.

## Les axes, et tu déclareras lesquels tu as exercés

1. **Ce que le split a pu perdre.** Chaque AC de la parente (1-12, 19-21) se retrouve-t-il dans la b1 ou la
   b2, sans trou ni doublon contradictoire ? Un renvoi de numéro d'AC resté faux après renumérotation ?
2. **Le module `download.ts` sans migration (AC 9).** Crée-t-il un défaut qu'un gate verrait : `lint`,
   `svelte-check`, un test de duplication, un import croisé ? Les tests « recopiés sans modification » des cas
   `parseContentDispositionFilename` sont-ils recopiables tels quels (imports, helpers `fakeJwt`, `authState`) ?
   Le test direct de `triggerDownload` est-il écrivable dans jsdom (`vi.stubGlobal('URL', …)` écrase-t-il le
   constructeur `URL` dont d'autres modules ont besoin) ?
3. **Les gardes i18n, par l'exécution (AC 11).** Les valeurs 11 / 44 / 37 et la ligne de
   `SITES_GABARIT_ATTENDUS` : **que rendent réellement les extracteurs** pour la forme de code prescrite ?
   Construis une **copie jetable** dans le scratchpad pour l'observer — jamais dans le dépôt. Le préfixe
   `audit-log-entity-` ajouté à `MOTIFS_DYNAMIQUES` entre-t-il en conflit avec les 12 clés `audit-log-*` de
   la 25-1c-a (orphelines, `lint-i18n-ownership`, préfixes « entièrement demandés ») ?
4. **Les corrections M2-M4 contre le contrat** : « jours UTC » est-il exact au regard des bornes de la
   25-1c-a ? Le vidage d'`entityId` et son rejet à la lecture couvrent-ils le bouton « Réinitialiser » et le
   retour arrière du navigateur ? Le test de l'URL de `getBlob` est-il écrivable avec le patron cité ?
   `RESULT_TOO_LARGE` : `parseErrorResponse` est-il bien le chemin de `getBlob` (lis `requestRaw`) ?
5. **L'E2E (AC 13)** : chaque scénario est-il constructible avec les helpers réels ? Le rôle, le mot de passe,
   la connexion d'un utilisateur créé par l'API, `clearAuthStorage` ? Le scénario 4 en jour UTC tient-il
   si l'entrée est créée à 23:59:59 UTC et lue après minuit ?
6. **Cohérence interne, décomptes, références de lignes** — recompte, ne relis pas. Grep `\b28\b` et `\b29\b`.

## Ce que tu rends

- **Les findings**, chacun avec sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact, **la
  commande ou l'extrait qui l'établit**, et le correctif. Pour un scénario, montre que **son état de départ
  est atteignable**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Ne qualifie jamais de
  « vérifié » ce que tu n'as pas exécuté.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante** — nommément `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, tout
`git commit`/`push`/`checkout`/`add`/`stash`/`reset`/`rebase`, `npm install`, `npm run build`,
`sqlx migrate`, `cargo test`/`cargo nextest`. Autorisés : `npm run check`, `npx vitest run` en lecture, et
une copie jetable de fichiers **dans le scratchpad**
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/p3-b1/`.
