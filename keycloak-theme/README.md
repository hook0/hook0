# Hook0 Example Keycloak theme

## Build the theme extension

```
./build.sh
```

## Theme Development Workflow

Build this theme `.jar` file with:

```bash
# build the theme and wrap it in a .jar file
mvn package

# move theme to /tmp/theme/ folder for futur use by keycloak
mkdir -p /tmp/theme && cp -v target/*.jar /tmp/theme/
```

Then start Keycloak IAM (in single-node mode) locally through docker and use the host `/tmp/theme` folder as Keycloak deployment directory using the `watch.sh` script

```bash
./watch.sh
```

Connect to Keycloak console [http://localhost:8080](http://localhost:8080), click on `Themes` tab, and select `hook0` in front of `Login Theme`.

## More :

If you want to activate the login, email or other theme, open :
```bash
src/main/resources/META-INF/keycloak-themes.json
```
change the type to put the themes you want :
```bash
{
    "themes": [{
        "name" : "hook0",
        "types": [ "login", "email" ]
    }]
}
```


## Resources

- https://github.com/layoutHook0-iam/keycloak-cloud-iam-theme
- https://www.baeldung.com/spring-keycloak-custom-themes
- https://github.com/InseeFrLab/keycloakify
