<#import "layout-hook0.ftl" as layout>
<@layout.registrationHook0Layout displayMessage=!messagesPerField.existsError('username') displayInfo=(realm.password && realm.registrationAllowed && !registrationDisabled??); section>
    <#if section = "header">
        ${msg("signInSSO")}
    <#elseif section = "form">
        <div id="kc-form">
            <div id="kc-form-wrapper">
                <#if realm.password>
                    <form id="kc-form-login" onsubmit="login.disabled = true; return true;" action="${url.loginAction}"
                          method="post">
                        <input>
                        <#if !usernameHidden??>
                            <div class="${properties.kcFormGroupClass!}">
                                <label for="username"
                                       class="${properties.kcLabelClass!}">${msg("organizationName")}</label>

                                <input tabindex="1" id="organization"
                                       aria-invalid="<#if messagesPerField.existsError('username')>true</#if>"
                                       class="${properties.kcInputClass!}" name="organization"
                                       type="text" autofocus autocomplete="off"/>

                                <#if messagesPerField.existsError('username')>
                                    <span id="input-error-username" class="${properties.kcInputErrorMessageClass!}" aria-live="polite">
                                        ${kcSanitize(messagesPerField.get('username'))?no_esc}
                                    </span>
                                </#if>
                            </div>
                        </#if>

                        <div id="kc-form-buttons" class="${properties.kcFormGroupClass!}">
                            <input tabindex="4"
                                   class="${properties.kcButtonClass!} ${properties.kcButtonPrimaryClass!} ${properties.kcButtonBlockClass!} ${properties.kcButtonLargeClass!}"
                                   name="login" id="kc-login" type="submit" value="${msg("doLogIn")}"/>
                        </div>
                    </form>
                </#if>
            </div>
        </div>
        <div id="kc-sso-providers" class="${properties.kcFormSocialAccountSectionClass!}">
            <hr/>
            <br/>
            <div id="kc-registration-container">
                <div id="kc-registration">
                    <span><a tabindex="6" href="${url.loginRestartFlowUrl}">${msg("signInWithAccount")}</a></span>
                </div>
            </div>
        </div>
    </#if>
</@layout.registrationHook0Layout>
