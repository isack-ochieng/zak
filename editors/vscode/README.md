# ZAK Language Support for VS Code

This is the first declarative editor extension for ZAK.

## Features

- Associates `.zak` files with the `zak` language ID.
- Syntax highlighting for the currently specified ZAK 0.1 syntax.
- Line comments, bracket matching, auto-closing and indentation.
- Snippets for `main`, `print`, `let` and `var`.
- A ZAK language icon for supported VS Code file icon themes.

This extension does not include the ZAK compiler or a language server yet. It does not currently provide semantic diagnostics, IntelliSense, go-to-definition, refactoring or debugging.

## Test locally

1. Open this directory in VS Code.
2. Press F5 to launch an Extension Development Host.
3. Create or open a file ending in `.zak`.
4. Confirm the status bar shows **ZAK** and the editor highlights the source.

## Package as VSIX

Install Node.js only if you are packaging or publishing the VS Code extension. From this directory:

```bash
npx --yes @vscode/vsce package
```

This produces a `.vsix` file. Install it with:

```bash
code --install-extension zak-language-support-0.1.0.vsix
```

The end user does not need npm to use the ZAK language or this packaged extension.

## Publishing

The extension can be published to the Visual Studio Code Marketplace after the publisher account is registered and the package is validated. The current publisher ID in `package.json` is the proposed project publisher name; Marketplace publication requires control of that publisher identity.

## Future editor integration

The next stage is a ZAK Language Server (LSP) that will provide compiler-backed diagnostics, completion, hover information, symbol navigation and formatting. Debugger integration will follow once the compiler has a stable debug-information and runtime model.
