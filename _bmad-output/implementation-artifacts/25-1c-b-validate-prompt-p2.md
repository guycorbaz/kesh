# Prompt — passe 2 de `bmad-create-story validate`, Story 25-1c-b

*Versionné le 2026-09-15. Une lentille en contexte frais (Opus), orthogonale aux deux lentilles de la
passe 1 (Sonnet, Haiku).*

Tu es un **valideur adversarial** en contexte frais. Ton objet est le fichier
`_bmad-output/implementation-artifacts/25-1c-b-journal-audit-ecran.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-1c-b-journal-audit-ecran`. Ta mission n'est pas
d'approuver : c'est de **trouver ce qui ferait échouer, dévier ou mentir** l'implémentation qui suivra.

⛔ **Rien ne se croit sur parole** — ni la spec, ni son Change Log, ni les vérifications de la passe 1.

⚠️ **Ne conteste PAS les arbitrages du Project Lead** ni le **contrat de la route** fixé par
`25-1c-a-journal-audit-route.md` (spec validée). Conteste la mise en œuvre côté écran.

## Où regarder d'abord

La passe 1 a réécrit l'AC 9 (trois sites de `parseContentDispositionFilename`, fusion des tests), l'AC 3
(patron restreint à la carte, repli sur code inconnu, « Facture importée ») et des références. **Ta base
de comparaison est le commit de la spec** : `git diff 5ea84b05 -- _bmad-output/implementation-artifacts/25-1c-b-journal-audit-ecran.md`.
Nomme-la dans ton rapport.

## Les axes, et tu déclareras lesquels tu as exercés

1. **La remédiation de la passe 1.** Le tableau des trois sites de l'AC 9 est-il complet — un **quatrième**
   consommateur (import, ré-export, test, composant Svelte) ? La fusion des tests des deux copies est-elle
   faisable sans perdre un cas ? Rejoue `npx vitest run` sur les fichiers concernés pour établir la base.
2. **Les gardes i18n, par l'exécution.** La forme prescrite pour `entityTypeLabel` (carte + appel
   `i18nMsg` à gabarit, repli code brut sans `i18nMsg`) : **que comptera exactement `i18n-keys.test.ts`** —
   `sitesGabarit`, `SITES_GABARIT_ATTENDUS`, `sitesTotal`, `MOTIFS_DYNAMIQUES`, `CARDINALITES` ? Et
   `i18n-libelle-en-dur.test.ts` : une fonction `entityTypeLabel` qui **rend un littéral** (le code brut)
   est-elle une « candidate » ? Lis les extracteurs réels, et si besoin construis une **copie jetable** du
   fichier dans le scratchpad pour observer ce que les compteurs rendent — jamais dans le dépôt.
3. **L'écran contre le contrat** : pagination, `entityId` désactivé sans type, message `RESULT_TOO_LARGE`,
   heure locale à l'écran / UTC au CSV ; un état que la spec n'a pas prévu (session expirée, 403 d'un rôle
   rétrogradé en cours de session, route 25-1c-a absente) ?
4. **L'E2E (AC 20)** : chaque scénario est-il constructible avec les helpers réels (`authedApiContext`,
   création d'un utilisateur Consultation, `seedTestState`) ? Le `seed` E2E crée-t-il des entrées d'audit
   parasites qui fausseraient les assertions ? `workers: 1` et la pollution d'état documentée dans
   `docs/testing.md` § *Les échecs attendus* sont-ils pris en compte ?
5. **Les manuels, le README, le glossaire (AC 13-18)** : la réécriture prescrite est-elle exacte ? Existe-t-il
   un site non nommé — manuels (PDF aplati : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, souligné LaTeX
   `\_`), `README.md`, `website/`, catalogues i18n (apostrophe typographique) ?
6. **Cohérence interne, décomptes, périmètre** — recompte, ne relis pas.

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
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/`.
