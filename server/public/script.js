// Isso obviamente não deve ficar aqui. TODO: Tela de login
const username = "nauzoo";
const password = "0000";

fetch('/login', {
    method : 'POST',
    credentials: 'include',
    headers: {
        'Content-Type' : 'application/json'
    },
    body : JSON.stringify({ username, password })

}).then(response => {
    if (!response.ok) throw new Error("User not found.");
    console.log("Login feito com sucesso! Cookie guardado.");
    return response;
}).catch(erro => console.error(erro))


const fileName = "test_files/polish-toilet.gif";

fetch(`/files/${fileName}`, {
    method: 'GET',
    credentials: 'include' 
})
.then(response => {
    if (!response.ok) {
        throw new Error("Acesso negado. Você está logado?");
    }
    // Como é um arquivo (e não JSON), transformamos em um Blob (Pacote de dados binários)
    return response.blob(); 
})
.then(blob => {
    // Código temporário...
    const urlLocal = URL.createObjectURL(blob);
    
    const imagemHTML = document.createElement("img");
    imagemHTML.src = urlLocal;
    imagemHTML.style.maxWidth = "100%"; // Só para não quebrar o layout
    
    document.body.appendChild(imagemHTML);
})
.catch(erro => {
    console.error("Erro ao buscar arquivo:", erro);
});