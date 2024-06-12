<div align="center">

  <img src="wireguard.svg" alt="logo" width="200" height="auto" />
  <h1>Rust Wireguard VPN Peers Manager</h1>
  
</div>

<!-- Table of Contents -->
# :notebook_with_decorative_cover: Table of Contents

- [About the Project](#star2-about-the-project)
  * [Tech Stack](#space_invader-tech-stack)
  * [Features](#dart-features)
  * [Environment Variables](#key-environment-variables)
- [Getting Started](#toolbox-getting-started)
  * [Prerequisites](#bangbang-prerequisites)
  * [Installation](#gear-installation)
- [Usage](#eyes-usage)

<!-- About the Project -->
## :star2: About the Project

<!-- TechStack -->
### :space_invader: Tech Stack

<details>
  <summary>Server</summary>
  <ul>
    <li><a href="https://www.rust-lang.org/">Rust</a></li>
    <li><a href="https://actix.rs/">Actix</a></li>
    <li><a href="https://github.com/zarvd/wiretun/">Wiretun</a></li>
  </ul>
</details>

<details>
<summary>Database</summary>
  <ul>
    <li><a href="https://www.postgresql.org/">PostgreSQL</a></li>
  </ul>
</details>

<!-- Features -->
### :dart: Features

- Wireguard Server
- Signup & Login
- Handle wireguard peers
- Add new allowed ips

<!-- Env Variables -->
### :key: Environment Variables

To run this project, you will need to add the following environment variables to your .env file

```
SERVER_ADDR=127.0.0.1:8080
PG.USER=username
PG.PASSWORD=password
PG.HOST=127.0.0.1
PG.PORT=5432
PG.DBNAME=db
PG.POOL.MAX_SIZE=16
```
<!-- Getting Started -->
## :toolbox: Getting Started

<!-- Prerequisites -->
### :bangbang: Prerequisites

First you will need to setup the database with the schema in sql/schema.sql

```shell
psql -f sql/schema.sql db
```
<!-- Installation -->
### :gear: Installation

Compile and run project

```shell
cargo run
```

The Postman Collection : [Download Rust VPN Postman Collection](VPN.postman_collection.json)

