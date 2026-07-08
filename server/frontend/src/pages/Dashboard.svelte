<!--
Dash board Containing the file grid and its' functionalities 
-->>

<script lang="ts">
    import { onMount } from 'svelte';

    interface file {id: number, name: string}

    let files:file[] = $state([]);
    let currentPage = $state(1);     
    let isLoading = $state(false);  
    let hasMoreFiles = $state(true);

    let batchAmount = 20;

    let listEndWatcher:Element;

    async function fetchFiles() {
        if (isLoading || !hasMoreFiles) return;
        
        isLoading = true;

        try {
            const response = await fetch(
                `/api/files?page=${currentPage}&amount=${batchAmount}`, {
                credentials: 'include' // <- needed to verify the browser's credentials
            });
            
            if (!response.ok) {
                console.error("A API retornou um erro:", response.status);
                hasMoreFiles = false; // Para de tentar buscar
                return; // Aborta a função antes de quebrar o Javascript
            }

            const fileBatch = await response.json();

            if (fileBatch.length === 0) { // empty files list -> no more files
                hasMoreFiles = false;
            } else {
                files = [...files, ...fileBatch]; // Self note: This weird notations means files += filesBatch
                // Svelte's reactivity needs an attribution in order to update the state of this objecct
                currentPage += 1;
            }
        } catch (erro) {
            console.error("Erro ao carregar arquivos:", erro);
        } finally {
            isLoading = false;
        }
    }

    onMount(() => {
        const observer = new IntersectionObserver((entries) => {
            const watcher = entries[0];
            // isIntersecting is true when the div enters the browser's viewport
            if (watcher.isIntersecting) {
                fetchFiles();
            }
        });

        if (listEndWatcher) {
            observer.observe(listEndWatcher);
        }

        // Cleans the memory when the user leaves the page
        return () => observer.disconnect();
    });
</script>

<main>
    <h2>Meus Arquivos</h2>

    <!-- code for a basic reactive mansory grid-->
    <div class="columns-2 sm:columns-2 lg:columns-3 py-10 md:py-20 gap-4">
        {#each files as a_file}
            <div class="mb-4 break-inside-avoid">
                <img class="w-full object-cver rounded-lg" src={`/api/files/test_files/${a_file.id}`} alt={a_file.name} />
            </div>
        {/each}
    </div>

    <div bind:this={listEndWatcher} class="watcher">
        {#if isLoading}
            <p>Carregando mais arquivos...</p>
        {:else if !hasMoreFiles}
            <p>Você chegou ao fim! Não há mais arquivos.</p>
        {/if}
    </div>
</main>

<style>
    @import "tailwindcss";

    .watcher {
        text-align: center;
        padding: 2rem;
        margin-top: 1rem;
        
        min-height: 50px; 
    }
</style>