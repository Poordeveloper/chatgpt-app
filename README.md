<div align="center">
<img src="./public/favicon.svg" alt="icon"/>

<h1 align="center">A ChatGPT App for all platforms</h1>

<!-- English / [简体中文](./docs/README-zh-CN.md) / [繁體中文](./docs/README-zh-TW.md) / [Deutsch](./docs/README-de-DE.md) / [Français](./docs/README-fr-FR.md) / [Italiano](./docs/README-it-IT.md) / [한국어](./docs/README-kr-KR.md) -->

Built with <b>Rust</b> + <b>Tauri</b> + <b>Vue</b> + <b>Axum</b>

 ![play](https://user-images.githubusercontent.com/1399893/236070870-40443458-687b-4532-abc9-5b2265a298d7.gif)
  
![cover](./docs/images/screenshot.png)

</div>

# Prerequisite
- [nodejs](https://nodejs.org/)
- [pnpm](https://pnpm.io/)
- [rust](https://rustup.rs/)
- [Android / iOS sdk](https://next--tauri.netlify.app/next/guides/getting-started/prerequisites/)

# Run Desktop

`pnpm tauri dev`

![desktop](docs/images/desktop.png)

# Run Web

```
# run backend
cargo run --bin server
# start frontend dev
pnpm dev
```

## Environment Variables (for web backend)

### `OPENAI_API_KEY` (required)
Your openai api key.

### `PROXY`
socks / https / http proxy server if you have problem to access OpenAI api server directly. e.g.
`socks5://myusername:mypassword@localhost:1080`

### `API_REVERSE_PROXY`
https://github.com/transitive-bullshit/chatgpt-api#reverse-proxy

### `AUTH_SECRET_KEY`
Key used to access your web

### `OPENAI_API_MODEL`
Use `gpt-3.5-turbo` by default

### `TIMEOUT_MS`
Timeout of OpenAI api request

# Run Android / iOS

```
pnpm tauri ios dev
```

# Thanks
- The initial frontend code was copied from [Chanzhaoyu/chatgpt-web](https://github.com/Chanzhaoyu/chatgpt-web)
- Inspired by [Awesome ChatGPT Prompts](https://github.com/f/awesome-chatgpt-prompts) and [lencx/ChatGPT](https://github.com/lencx/ChatGPT)
