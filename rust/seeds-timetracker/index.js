let auth0Client = null;

window.init_auth = async (domain, client_id) => {
    console.info("init auth")
    auth0Client = await auth0.createAuth0Client({
        domain,
        clientId: client_id,
        authorizationParams: {
            redirect_uri: window.location.origin
        },
        cacheLocation: "localstorage"
    });
    console.info("\tinit auth: done")

    const query = window.location.search;
    if (query.includes("code=") && query.includes("state=")) {
        await auth0Client.handleRedirectCallback();
    }

    if (await auth0Client.isAuthenticated()) {
        return await auth0Client.getUser();
    }
}

window.redirect_to_sign_up = async () => {
    console.log("Sign UP!");
    await auth0Client.loginWithRedirect({
        screen_hint: "signup",
    });
}

window.redirect_to_log_in = async () => {
    await auth0Client.loginWithRedirect();
}

window.logout = () => {
    auth0Client.logout();
}
