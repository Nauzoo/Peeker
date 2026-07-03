<script>
    import { Router, Route, Link } from "svelte-routing";
    
    import Login from "./pages/Login.svelte";
    import Register from "./pages/Register.svelte";
    //import Dashboard from "./pages/Dashboard.svelte";

    export let url = "";
</script>

<Router {url}>
    
    <nav style="margin-bottom: 20px; padding: 10px; background: #eee;">
        <Link to="/">Início (Dashboard)</Link> | 
        <Link to="/login">Login</Link> | 
        <Link to="/register">Registro</Link>
    </nav>

    <Route path="/login" component={Login} />
    <Route path="/register" component={Register} />
    <!-- <Route path="/" component={Dashboard} /> -->
    
</Router>