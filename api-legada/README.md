# API de Mapa de Coletivos de Música Eletrônica

## Sobre o Projeto

Este projeto "backend" consiste em uma API RESTful desenvolvida em Node.js para gerenciar um mapa de coletivos de música eletrônica. Utilizando o Express e outras tecnologias modernas, esta API fornece um serviço robusto para organizar e disponibilizar informações sobre coletivos de música eletrônica, eventos, artistas e locais.

## Versão

- **Versão Atual:** 1.0.0

## Funcionalidades Principais

- **Gerenciamento de Coletivos:** Adição, atualização, remoção e listagem de coletivos de música eletrônica.
- **Informações de Eventos:** Possibilidade de inserir e consultar eventos relacionados aos coletivos, incluindo locais, datas e artistas.
- **API Restful:** Interface RESTful para fácil integração com front-ends e outros sistemas.
- **Upload de Mídias:** Suporte para o upload de imagens ou arquivos de áudio relacionados aos coletivos e eventos, utilizando o `multer`.

## Tecnologias Utilizadas

- **Express:** Framework web para Node.js.
- **TypeORM:** ORM para TypeScript, facilitando a interação com o banco de dados Postgres.
- **SQLite3:** Banco de dados leve e eficiente para armazenar os dados da aplicação.
- **Yup:** Biblioteca para validação de esquemas, garantindo a integridade dos dados recebidos pela API.
- **CORS:** Middleware para habilitar CORS (Cross-Origin Resource Sharing).

## Como Executar

1. **Instalação das Dependências:**
   ```bash
   yarn install
   ```
2. **Executar em Modo de Desenvolvimento:**
   ```bash
   yarn run dev
   ```
   Isso iniciará o servidor em modo de desenvolvimento, utilizando `ts-node-dev` para reinício automático em caso de alterações no código.
   
3. **Execução de Comandos do TypeORM:**
   ```bash
   yarn run typeorm [comando]
   ```
   Substitua `[comando]` pelo comando desejado do TypeORM para gerenciar o banco de dados.

## Licença

- **Licenciado sob a Licença MIT.**

## Suporte e Contribuições

- Para suporte, abra uma issue no repositório do projeto.
- Contribuições são bem-vindas. Para contribuir, faça um fork do projeto e abra um Pull Request com suas alterações.