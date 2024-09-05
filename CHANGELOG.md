## v0.5.0 (2024-09-05)

### Feat

- **all**: adding python library wrapper

### Fix

- **python**: refactoring python client
- **cargo**: adding vendored openssl to help with build
- **python-client**: fixing paged search results to use a PyList instead

### Refactor

- **python**: adding custom python modules to possibly help with documentation
- **models**: removing unnecessary python classes
- **models**: saving some changes in prep of python library

## v0.4.0 (2024-08-12)

### Feat

- **client**: changed all resources so that the URLs include the server's base URL

### Refactor

- **tokens**: cleanup dead code

## v0.3.0 (2024-08-12)

### Feat

- **client**: adding post image and post thumbnail downloading methods

### Fix

- **client**: fixing API path for file downloads
- **tokens**: fixing token encoding
- **models**: fixing parsing of snapshot data

## v0.2.3 (2024-08-11)

### Fix

- **client**: fixing reverse image search

### Refactor

- **main**: fixing some stuff for clippy

## v0.2.2 (2024-08-11)

### Fix

- **client**: fixes for user and token CRUD operations

## v0.2.1 (2024-08-10)

### Fix

- **client**: fixing tokens to not be encoded wholesale
- **client**: fixing pool CRUD operations

## v0.2.0 (2024-08-10)

### Fix

- **client.rs**: fixing file uploads requiring file name

## v0.1.1 (2024-08-10)
