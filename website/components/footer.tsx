import { FooterLinks } from "./footer-links";

import { socialMedia } from "@/app/data";

const Footer = () => {
  return (
    <footer className="w-full pb-5">
      {/* Footer links */}
      <FooterLinks />

      {/* Separator */}
      <div className="w-full h-px bg-gray-400" />

      {/* Footer */}
      <div className="flex mt-8 md:flex-row flex-col justify-between items-center">
        <div className="flex flex-col md:text-base text-sm md:font-normal font-light">
          <p>© Hook0. All rights reserved.</p>
          <p>
            Hook0 is made by an European French Tech company from Nantes,
            France.
          </p>
          <p>100% bootstraped, no VCs, we are here to stay.</p>
        </div>

        <div className="flex items-center md:gap-3 gap-6 mt-6 md:mt-0">
          {socialMedia.map((info) => (
            <div
              key={info.link}
              className="w-10 h-10 cursor-pointer flex justify-center items-center backdrop-filter backdrop-blur-lg saturate-180 bg-opacity-75 bg-black-200 rounded-lg border border-black-300"
            >
              <a href={info.link} rel="noopener noreferrer" target="_blank">
                <img alt="icons" height={20} src={info.img} width={20} />
              </a>
            </div>
          ))}
        </div>
      </div>
    </footer>
  );
};

export default Footer;
