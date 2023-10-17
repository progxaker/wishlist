FROM nginx:1.25.2-bookworm

COPY index.html /usr/share/nginx/html
COPY js/ /usr/share/nginx/html/js/
COPY style/ /usr/share/nginx/html/style/
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 8080
