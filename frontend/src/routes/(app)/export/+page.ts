// Story 9-2b — Page export global ZIP.
//
// `ssr = false` : la page nécessite un browser réel pour déclencher le
// download (`URL.createObjectURL` + `<a download>`), cohérent
// reports/+page.ts pattern.
//
// Pas de `load()` initial — pas de données à pré-charger (le bouton
// déclenche tout le pipeline backend).

export const ssr = false;
