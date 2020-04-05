import { Subject } from 'rxjs';

export async function invoke(arg: any) {
    const s = new Subject();
    s.next()
    // invoke function is supplied automagically by web-view
    // typescript doesn't know this so mark it as "any" to get around it
    const external: any = window.external;
    external.invoke(JSON.stringify(arg));
    let p = new Promise((resolve, reject) => {

    });
}