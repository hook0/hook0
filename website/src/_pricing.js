const destination = {
  CLOUD: 'cloud',
  ONPREMISE: 'onpremise',
};

const QUERYSTRING_KEY = 'pricing.destination';

document.addEventListener('DOMContentLoaded', function () {
  const container = document.querySelectorAll('.pricing')[0];

  document.querySelectorAll('.pricing-switcher a').forEach(function (link) {
    link.addEventListener('click', function (e) {
      e.preventDefault();
      e.stopImmediatePropagation();
      display(container.className.match(/deploy\-(.+)/)[1] === destination.CLOUD ? destination.ONPREMISE : destination.CLOUD);
    });
  });

  function display(newModel) {
    container.className = container.className.replace(/deploy\-(.+)/, `deploy-${newModel}`);
    const url = new URL(window.location);
    url.searchParams.set(QUERYSTRING_KEY, newModel);
    history.pushState({}, null, url);
  }

  const url = new URL(window.location);
  display(url.searchParams.get(QUERYSTRING_KEY) || destination.CLOUD);
});
