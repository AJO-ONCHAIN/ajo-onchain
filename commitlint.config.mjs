/**
 * Conventional Commits, enforced by husky's commit-msg hook.
 *
 * The scopes below match the top-level areas of the repository. They are not
 * bureaucracy: `git log --oneline -- contracts/` is how a reviewer or an
 * auditor reconstructs what happened to the contracts, and a scope makes that
 * readable without opening every diff.
 */
export default {
  extends: ["@commitlint/config-conventional"],
  rules: {
    "scope-enum": [
      2,
      "always",
      [
        "contracts",
        "vault",
        "factory",
        "auction",
        "registry",
        "shared",
        "indexer",
        "web",
        "scoring",
        "bindings",
        "docs",
        "audit",
        "ci",
        "deps",
        "repo",
      ],
    ],
    // Room for a real sentence in the body, which is where the reasoning
    // belongs. The subject line stays short.
    "body-max-line-length": [1, "always", 100],
  },
};
