const BASE_PATH = '/dinos';

// helpers

// https://gist.github.com/antonioaguilar/6135f84658328d399ed656ba3169e558
function UUID() {
    var d = new Date().getTime();
    var uuid = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
      var r = (d + Math.random() * 16) % 16 | 0;
      d = Math.floor(d / 16);
      return (c === 'x' ? r : (r & 0x3 | 0x8)).toString(16);
    });
    return uuid;
}

// based on https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch
async function api( method, data = {}) {
    let url = BASE_PATH;
    if( ! data.id ) {
        data.id = UUID();
    } else {
        url += `/${data.id}`;
    }

    // we must support this on the backend
    data.weight = parseInt( data.weight, 10 );
    const response = await fetch(url, {
      method,
      cache: 'no-cache',
      headers: {
        'Content-Type': 'application/json'
      },
      redirect: 'follow',
      referrerPolicy: 'no-referrer',
      body: JSON.stringify(data)
    });

    if( ! response.ok ) throw new Error('Error persistinf dinos');
  }