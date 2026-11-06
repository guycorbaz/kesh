# Prompt — revue de code P2 ciblée de la fiche `v0174-correctifs-3` (les écrans), une lentille, Sonnet

Passe **ciblée** : une seule lentille, en contexte frais, braquée sur **la seule remédiation de la revue P1** de la fiche 3
de Kesh (0.17.4, sans migration). Modèle : **Sonnet**. Langue : français. Rapport (le **seul** fichier que tu écris) :
`/home/gcorbaz/devel/kesh-gate-logs/v0174c-review-p2-3.md`.

Worktree `/home/gcorbaz/devel/kesh-0174c`, branche `story/v0174-correctifs`. La remédiation, aplatie :
`git -C /home/gcorbaz/devel/kesh-0174c show f3b5d5b1 -- frontend crates/kesh-i18n` et `git show ae37e7fb`. Les findings P1 visés :
`/home/gcorbaz/devel/kesh-gate-logs/v0174c-review-p1-3-{B,E,A}.md` — MEDIUM A-1 (captures #888 et #817), LOW B-3 (it-CH au
tu, guillemets en-CH), E-1 (casse de l'avis), E-3 (fuseau de `reminder-sent-at.test.ts`), A-2 (portée réelle de MC-889-a et
MC-889-c au Dev Agent Record). Fiche : `_bmad-output/implementation-artifacts/v0174-correctifs-3.md` (entrée « Remédiation
de la revue P1 »). Pièces, lecture seule : **les captures** `kesh-gate-logs/v0174c/captures-r3/v0174c-*.png` (ouvre-les :
`888-lot-haut-de-liste`, `888-lot-pied-de-liste`, `817-contact-pays-inconnu-avis`, thème clair), `mut/MC-817-casse.log`,
`mut/mutations.log`, `vitest-final3.log`, `check-final3.log`, `e2e-full-3.log`, `e2e-full-4.log`, `gate-front3.log`.

## Interdits absolus

- **Lecture seule** : aucun fichier hormis ton rapport, aucun fichier temporaire, **aucune redirection**. **Aucun `rm`**,
  aucun `git` qui écrit.
- **Rien qui compile ou exécute** : ni `npm`/`npx`/`node`/`vitest`/`playwright`, ni `cargo`, ni `python`, ni `make`, ni
  `docker`, ni `curl`, ni `claude`, ni `sed -i`, ni **`scripts/prepare-release.sh`**, ni aucun script de `kesh-gate-logs`.
  **Aucune commande `gh`.**
- Permis : `git diff/show/log/grep`, `grep`, `sed -n`, `awk` (sans redirection), `cat`, `ls`, `wc`, la lecture d'images.
- Toute affirmation de présence ou d'absence : une sortie `grep -nF` (ou `sed -n`) copiée.

## Axes

1. **Les captures montrent-elles ce que la fiche demande ?** Le pied de la liste d'un lot de plus de 200 factures et leur
   nombre ; l'avis du pays inconnu **en entier**. Le texte injecté dans la page par la spec (« [capture E2E] … ») pour
   écrire le nombre de cases est-il honnête (pris de `boxes.count()` après l'assertion `toHaveCount(open.total)`) ?
2. **La spec E2E modifiée** (`v0174-correctifs.spec.ts`) : numéros `LOT888-…-001…205`, `scrollIntoViewIfNeeded`,
   `toBeInViewport`, `setViewportSize` — un défaut neuf (sélecteur sur un libellé traduit, garde
   `e2e-selecteurs-traduits.test.ts` ; état laissé aux specs suivantes) ? Vert au run 4 (`e2e-full-4.log`) ; et `invoices.spec.ts:501`, corrigé en `ae37e7fb` (date du poste) ?
3. **L'avis insensible à la casse** (`contacts/+page.svelte`) : comparaison égale à la tolérance du serveur
   (`address_input.rs`, `trim` + majuscules) ; le test neuf (« su ») ; la mutation MC-817-casse tuée.
4. **i18n** : it-CH « Correggilo » au registre du fichier ; en-CH guillemets droits comme le reste du fichier ; repli fr-CH
   inchangé ; gardes vertes.
5. **E-3** : `pinTimeZone('UTC')` dans le `describe` de #259 ; **A-2** : le Dev Agent Record dit la portée réelle des
   campagnes MC-889-a et MC-889-c.

## Rapport

Findings (sévérité, fichier:ligne, preuve copiée, remède) ; liste des axes **exercés** et **non exercés** ; décompte
recompté.
