<#import "layoutHook0.ftl" as layout>
<@layout.layoutHook0 ; section>
    <#if section = "text">
        ${kcSanitize(msg("emailVerificationBodyHtml",link, linkExpiration, realmName, linkExpirationFormatter(linkExpiration)))?no_esc}
    </#if>
</@layout.layoutHook0>
