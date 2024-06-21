import { footerItems } from "@/app/data";

export const FooterLinks = () => {
  const isExternalLink = (link: string) => /^https?:/.test(link);

  return (
    <div className="flex justify-center gap-6 md:gap-16 my-16">
      {footerItems.map((info) => (
        <div key={info.category} className="flex flex-col">
          <h3 className="text-lg font-bold mb-4">{info.category}</h3>
          <ul>
            {info.items.map((item) => (
              <li key={item.name} className="mb-1">
                <a
                  href={item.link}
                  rel={
                    isExternalLink(item.link)
                      ? "noopener noreferrer"
                      : undefined
                  }
                  target={isExternalLink(item.link) ? "_blank" : "_self"}
                >
                  {"icon" in item && item.icon && (
                    <item.icon className="inline mr-2" />
                  )}
                  {item.name}
                </a>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </div>
  );
};
